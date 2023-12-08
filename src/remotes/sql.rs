use anyhow::Result;
use mysql::prelude::Queryable;
use mysql::Error;
use mysql::Pool;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub constraint_name: String,
    pub column_name: String,
    pub referenced_table_name: String,
    pub referenced_column_name: String,
}

#[derive(Debug, Clone)]
pub struct IndexKey {
    pub constraint_name: String,
    pub column_name: String,
}

impl IndexKey {
    fn to_prisma_directive(&self) -> String {
        format!(
            "@index([{}]), map: {}",
            self.column_name, self.constraint_name
        )
    }
}

#[derive(Debug, Clone)]
pub struct UniqueKey {
    pub constraint_name: String,
    pub column_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Key {
    Foreign(ForeignKey),
    Index(IndexKey),
    MultiIndex(Vec<IndexKey>),
    Unique(Vec<UniqueKey>),
}

#[derive(Debug)]
pub struct TableKeys {
    pub keys: Vec<Key>,
}

impl From<Vec<ForeignKeyInformation>> for TableKeys {
    fn from(rows: Vec<ForeignKeyInformation>) -> Self {
        let mut keys: Vec<Key> = Vec::new();
        let unique_constraint_data = group_unique_constraints(&rows);
        for (constraint_name, column_names) in unique_constraint_data {
            keys.push(Key::Unique(vec![UniqueKey {
                constraint_name,
                column_names,
            }]));
        }
        for row in rows {
            // Need to add other key types here
            if row.referenced_column_name.is_some() && row.referenced_table_name.is_some() {
                keys.push(Key::Foreign(ForeignKey {
                    constraint_name: row.constraint_name.unwrap(),
                    column_name: row.column_name.unwrap(),
                    referenced_table_name: row.referenced_table_name.unwrap(),
                    referenced_column_name: row.referenced_column_name.unwrap(),
                }));
            }
        }
        TableKeys { keys }
    }
}

fn group_multiindex_constraints(data: &Vec<ForeignKeyInformation>) -> HashMap<String, Vec<String>> {
    data.iter()
        .filter_map(|info| {
            if info.constraint_type.as_deref() == Some("UNIQUE") {
                info.constraint_name.clone().zip(info.column_name.clone())
            } else {
                None
            }
        })
        .fold(HashMap::new(), |mut acc, (constraint_name, column_name)| {
            acc.entry(constraint_name)
                .or_insert_with(Vec::new)
                .push(column_name);
            acc
        })
}

fn group_unique_constraints(data: &Vec<ForeignKeyInformation>) -> HashMap<String, Vec<String>> {
    data.iter()
        .filter_map(|info| {
            if info.constraint_type.as_deref() == Some("UNIQUE") {
                info.constraint_name.clone().zip(info.column_name.clone())
            } else {
                None
            }
        })
        .fold(HashMap::new(), |mut acc, (constraint_name, column_name)| {
            acc.entry(constraint_name)
                .or_insert_with(Vec::new)
                .push(column_name);
            acc
        })
}

type ForeignKeySQLResponse = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<usize>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<usize>,
    Option<usize>,
    Option<String>,
);

#[derive(Debug, Clone)]
struct ForeignKeyInformation {
    constraint_name: Option<String>,
    constraint_type: Option<String>,
    column_name: Option<String>,
    ordinal_position: Option<usize>,
    referenced_table_name: Option<String>,
    referenced_column_name: Option<String>,
    index_name: Option<String>,
    seq_in_index: Option<usize>,
    cardinality: Option<usize>,
    index_type: Option<String>,
}

impl From<ForeignKeySQLResponse> for ForeignKeyInformation {
    fn from(row: ForeignKeySQLResponse) -> Self {
        let (
            constraint_name,
            constraint_type,
            column_name,
            ordinal_position,
            referenced_table_name,
            referenced_column_name,
            index_name,
            seq_in_index,
            cardinality,
            index_type,
        ) = row;
        ForeignKeyInformation {
            constraint_name,
            constraint_type,
            column_name,
            ordinal_position,
            referenced_table_name,
            referenced_column_name,
            index_name,
            seq_in_index,
            cardinality,
            index_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Description {
    pub field: String,
    pub type_: String,
    pub null: String,
    pub key: String,
    pub default: Option<String>,
    pub extra: String,
}

type FieldDescriptionSQLResponse = (String, String, String, String, Option<String>, String);

impl std::fmt::Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "field: {}, type: {}, null: {}, key: {}, default: {:?}, extra: {}",
            self.field, self.type_, self.null, self.key, self.default, self.extra
        )
    }
}

impl From<FieldDescriptionSQLResponse> for Description {
    fn from(row: FieldDescriptionSQLResponse) -> Self {
        let (field, type_, null, key, default, extra) = row;
        Description {
            field,
            type_,
            null,
            key,
            default,
            extra,
        }
    }
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub description: Vec<Description>,
    pub keys: TableKeys,
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Table: {}", self.name)?;
        for description in &self.description {
            write!(f, "{}", description)?;
        }
        Ok(())
    }
}

struct ErrorReport {
    table: String,
    error: Error,
}

impl std::fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Table: {}, Error: {}", self.table, self.error)
    }
}

pub fn get_table_descriptions(url: &str) -> Result<Vec<Table>> {
    let pool = Pool::new(url)?;
    let mut conn = pool.get_conn().unwrap();
    let mut errors: Vec<ErrorReport> = vec![];
    let tables: Vec<String> = conn.query("SHOW TABLES").unwrap_or_else(|e| {
        errors.push(ErrorReport {
            table: String::from("All tables"),
            error: e,
        });
        vec![]
    });
    let mut all_descriptions: Vec<Table> = vec![];
    for table in tables {
        let descriptions: Vec<FieldDescriptionSQLResponse> = conn
            .query(format!("DESCRIBE {}", &table))
            .unwrap_or_else(|e| {
                errors.push(ErrorReport {
                    table: table.clone(),
                    error: e,
                });
                vec![]
            });
        let relations_query = format!(
            "
SELECT
    constraint_name,
    constraint_type,
    column_name,
    ordinal_position,
    referenced_table_name,
    referenced_column_name,
    index_name,
    seq_in_index,
    cardinality,
    index_type
FROM (
    SELECT
        TC.CONSTRAINT_NAME AS constraint_name,
        TC.CONSTRAINT_TYPE AS constraint_type,
        KCU.COLUMN_NAME AS column_name,
        KCU.REFERENCED_TABLE_NAME as referenced_table_name,
        KCU.REFERENCED_COLUMN_NAME as referenced_column_name,
        KCU.ORDINAL_POSITION AS ordinal_position,
        NULL AS index_name,
        NULL AS seq_in_index,
        NULL AS cardinality,
        NULL AS index_type
    FROM
        INFORMATION_SCHEMA.TABLE_CONSTRAINTS TC
    JOIN
        INFORMATION_SCHEMA.KEY_COLUMN_USAGE KCU
    ON
        TC.CONSTRAINT_NAME = KCU.CONSTRAINT_NAME
    JOIN
        INFORMATION_SCHEMA.COLUMNS C
    ON
        KCU.COLUMN_NAME = C.COLUMN_NAME
        AND KCU.TABLE_NAME = C.TABLE_NAME
        AND KCU.TABLE_SCHEMA = C.TABLE_SCHEMA
    WHERE
        TC.TABLE_NAME = '{}'
        AND C.TABLE_NAME IS NOT NULL
        AND TC.CONSTRAINT_TYPE NOT IN ('PRIMARY KEY', 'CHECK')
    UNION ALL
    SELECT
        NULL AS constraint_name,
        NULL AS constraint_type,
        COLUMN_NAME AS column_name,
        NULL as referenced_table_name,
        NULL as referenced_column_name,
        SEQ_IN_INDEX AS ordinal_position,
        INDEX_NAME,
        SEQ_IN_INDEX,
        CARDINALITY,
        INDEX_TYPE
    FROM
        INFORMATION_SCHEMA.STATISTICS
    WHERE
        TABLE_NAME = '{}'
) AS combined_data
ORDER BY
    constraint_name, ordinal_position;
    ",
            &table, &table
        );
        let relations: Vec<ForeignKeySQLResponse> =
            conn.query(relations_query).unwrap_or_else(|e| {
                errors.push(ErrorReport {
                    table: table.clone(),
                    error: e,
                });
                vec![]
            });
        let as_info: Vec<ForeignKeyInformation> = relations
            .into_iter()
            .map(ForeignKeyInformation::from)
            .collect();
        if descriptions.len() > 0 {
            all_descriptions.push(Table {
                name: table,
                description: descriptions.into_iter().map(Description::from).collect(),
                keys: TableKeys::from(as_info),
            });
        }
    }
    Ok(all_descriptions)
}

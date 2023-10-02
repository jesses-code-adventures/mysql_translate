use mysql::prelude::*;
use mysql::*;

#[derive(Debug, Clone)]
pub struct Description {
    pub field: String,
    pub type_: String,
    pub null: String,
    pub key: String,
    pub default: Option<String>,
    pub extra: String,
}

type Row = (String, String, String, String, Option<String>, String);

impl std::fmt::Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "field: {}, type: {}, null: {}, key: {}, default: {:?}, extra: {}",
            self.field, self.type_, self.null, self.key, self.default, self.extra
        )
    }
}

impl From<Row> for Description {
    fn from(row: Row) -> Self {
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
        let descriptions: Vec<Row> =
            conn.query(format!("DESCRIBE {}", table))
                .unwrap_or_else(|e| {
                    errors.push(ErrorReport {
                        table: table.clone(),
                        error: e,
                    });
                    vec![]
                });
        if descriptions.len() > 0 {
            all_descriptions.push(Table {
                name: table,
                description: descriptions.into_iter().map(Description::from).collect(),
            });
        }
    }
    println!("{:#?}", &all_descriptions);
    Ok(all_descriptions)
}

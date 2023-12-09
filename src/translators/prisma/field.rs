use crate::remotes::sql::Description;
use crate::translators::prisma::relation::Relation;
use crate::translators::prisma::unique_flag::UniqueFlag;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
pub struct Field {
    pub name: String,
    pub target_name_width: usize,
    pub field_type: String,
    pub target_field_type_width: usize,
    pub db_type_annotation: Option<String>,
    pub is_required: bool,
    pub is_array: bool,
    pub is_id: bool,
    pub relation: Option<Relation>,
    pub default: Option<String>,
    pub unique: Option<UniqueFlag>,
}

impl Field {
    fn new() -> Field {
        Field {
            name: String::new(),
            target_name_width: 0,
            field_type: String::new(),
            target_field_type_width: 0,
            db_type_annotation: None,
            is_required: false,
            is_array: false,
            is_id: false,
            relation: None,
            default: None,
            unique: None,
        }
    }
    pub fn parse_from_disk(field_str: &str) -> Option<Field> {
        if field_str.starts_with("@@") || field_str.starts_with("@relation") {
            panic!("uncaught directive {}", field_str);
        }
        let mut field_str_mut = field_str.to_string();
        let mut name = String::new();
        let mut field_type = String::new();
        let mut db_type_annotation: Option<String> = None;
        let mut relation: Option<Relation> = None;
        let mut functions: Vec<String> = vec![];
        let mut parsing_function = false;
        let mut function_buffer = String::new();
        let mut open_brackets = 0;
        let mut open_squirlys = 0;
        let mut open_parens = 0;
        let mut empty_skipped = 0;
        let mut is_array = false;
        let mut is_required = true;
        let mut is_id = false;
        let mut default: Option<String> = None;
        let mut unique: Option<UniqueFlag> = None;

        // Handle unique piece and remove it if it exists
        if field_str_mut.contains("@unique") {
            let pieces: Vec<&str> = field_str.split("@").collect::<Vec<&str>>()[1..].to_vec();
            let mut unique_piece = "@".to_string();
            unique_piece.push_str(
                pieces
                    .into_iter()
                    .filter(|piece| piece.contains("unique"))
                    .collect::<Vec<&str>>()
                    .pop()
                    .unwrap(),
            );
            field_str_mut = field_str_mut.replace(unique_piece.as_str(), "");
            unique = Some(UniqueFlag::from(unique_piece.as_str()));
        }

        // Iterate through the rest of the line as words
        let pieces: Vec<&str> = field_str_mut.split(" ").collect();
        for (i, piece) in pieces.into_iter().enumerate() {
            if piece.len() == 0 {
                empty_skipped = empty_skipped + 1;
                continue;
            }
            if i - empty_skipped == 0 {
                name.push_str(piece);
                continue;
            }
            if i - empty_skipped == 1 {
                field_type.push_str(piece);
                if piece.contains("[]") {
                    is_array = true;
                }
                if piece.contains("?") {
                    is_required = false;
                } else {
                    is_required = true;
                }
                continue;
            }
            if piece.contains("@id") {
                is_id = true;
                continue;
            }
            if piece.contains("@default") {
                default = Some(
                    piece
                        .strip_prefix("@default(")
                        .expect("default to be formatted properly")
                        .strip_suffix(")")
                        .expect("default to be formatted properly")
                        .to_string(),
                )
            }
            if piece.contains("@db.") {
                if db_type_annotation.is_some() {
                    panic!("more than one db type annotation found");
                } else {
                    db_type_annotation = Some(piece.to_string());
                }
            }
            if piece.contains("(") || piece.contains("[") || piece.contains("{") {
                parsing_function = true;
                open_parens = open_parens + piece.matches("(").count();
                open_brackets = open_brackets + piece.matches("[").count();
                open_squirlys = open_squirlys + piece.matches("{").count();
            }
            if parsing_function {
                let mut to_push = format!("{}", piece);
                if to_push.ends_with(",") {
                    to_push.push_str(" ")
                }
                let _ = &function_buffer.push_str(to_push.as_str());
            }
            if piece.contains(")") || piece.contains("]") || piece.contains("}") {
                open_parens = open_parens - piece.matches(")").count();
                open_brackets = open_brackets - piece.matches("]").count();
                open_squirlys = open_squirlys - piece.matches("}").count();
                if open_parens == 0 && open_brackets == 0 && open_squirlys == 0 {
                    parsing_function = false;
                    if function_buffer.clone().len() > 0 {
                        if function_buffer.starts_with("@db.") {
                            db_type_annotation = Some(function_buffer.clone());
                        } else if function_buffer.starts_with("@relation") {
                            relation = Some(Relation::from_string(function_buffer.clone()));
                        } else {
                            functions.push(function_buffer.clone());
                        }
                        function_buffer.clear();
                        continue;
                    }
                }
                continue;
            }
            continue;
        }
        if parsing_function {
            println!("Unterminated delimiter in {}", field_str_mut);
            return None;
        }
        if name == "}" {
            return None;
        }
        let field = Field {
            name,
            target_name_width: 0,
            field_type,
            target_field_type_width: 0,
            db_type_annotation,
            is_array,
            is_required,
            is_id,
            relation,
            default,
            unique,
        };
        Some(field)
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_target_name_length(&mut self, length: usize) {
        if length > self.target_name_width {
            self.target_name_width = length;
        }
    }
    fn set_field_type(&mut self, field_type: String) {
        self.field_type = field_type;
    }
    fn set_db_type_annotation(&mut self, db_type_annotation: String) {
        self.db_type_annotation = Some(db_type_annotation);
    }
    pub fn set_field_type_length(&mut self, length: usize) {
        if length > self.target_field_type_width {
            self.target_field_type_width = length;
        }
    }
    fn set_is_required(&mut self, is_required: bool) {
        self.is_required = is_required;
    }
    fn set_unique(&mut self, unique: Option<UniqueFlag>) {
        self.unique = unique;
    }
    fn set_is_id(&mut self, is_id: bool) {
        self.is_id = is_id;
    }
    pub fn as_text(
        &self,
        max_name_width: usize,
        max_field_width: usize,
        number_of_id_fields: usize,
    ) -> String {
        let mut text = String::new();
        text.push_str(&self.name);
        let spaces_to_add = max_name_width - self.name.len();
        for _ in 0..spaces_to_add {
            text.push_str(" ");
        }
        text.push_str(" ");
        text.push_str(&self.field_type);
        let spaces_to_add = max_field_width - self.field_type.len();
        for _ in 0..spaces_to_add {
            text.push_str(" ");
        }
        text.push_str(" ");
        if self.is_id && number_of_id_fields == 1 {
            text.push_str("@id ");
        }
        if self.unique.is_some() {
            text.push_str(self.unique.as_ref().unwrap().clone().as_text().as_str());
            text.push_str(" ");
        }
        if self.default.is_some() {
            text.push_str("@default(");
            text.push_str(self.default.as_ref().unwrap().as_str());
            text.push_str(") ");
            text.push_str(" ");
        }
        if self.db_type_annotation.is_some() {
            text.push_str(
                self.format_db_annotation()
                    .expect(self.db_type_annotation.clone().unwrap().as_str())
                    .as_str(),
            );
            text.push_str(" ");
        }
        if self.relation.is_some() {
            text.push_str(self.relation.clone().unwrap().as_text().as_str());
            text.push_str(" ");
        }
        text
    }
    fn get_field_type_from_db_type(&self) -> String {
        let db_type = self
            .db_type_annotation
            .clone()
            .unwrap_or("String".to_string());
        let mut resp = String::new();
        match db_type {
            _ if db_type.eq("tinyint(1)") => resp.push_str("Boolean"),
            _ if db_type.contains("tinyint") => resp.push_str("Int"),
            _ if db_type.contains("smallint") => resp.push_str("Int"),
            _ if db_type.contains("mediumint") => resp.push_str("Int"),
            _ if db_type.contains("int") => resp.push_str("Int"),
            _ if db_type.contains("bigint") => resp.push_str("Int"),
            _ if db_type.contains("float") => resp.push_str("Float"),
            _ if db_type.contains("double") => resp.push_str("Float"),
            _ if db_type.contains("decimal") => resp.push_str("Decimal"),
            _ if db_type.contains("date") => resp.push_str("DateTime"),
            _ if db_type.contains("time") => resp.push_str("DateTime"),
            _ if db_type.contains("timestamp") => resp.push_str("DateTime"),
            _ if db_type.contains("year") => resp.push_str("Int"),
            _ if db_type.contains("bool") => resp.push_str("Boolean"),
            _ if db_type.contains("json") => resp.push_str("Json"),
            _ => resp.push_str("String"),
        }
        if !self.is_required {
            resp.push_str("?");
        }
        resp
    }
    fn format_db_annotation(&self) -> Option<String> {
        let mut resp = String::from("@db.");
        //  This feels bad
        match self
            .db_type_annotation
            .clone()
            .unwrap_or_else(|| "".to_string())
            .as_str()
        {
            db_type if db_type.contains("tinyint") => resp.push_str("TinyInt"),
            db_type if db_type.contains("smallint") && db_type.contains("unsigned") => resp
                .push_str(
                    db_type
                        .replace("smallint unsigned", "UnsignedSmallInt")
                        .as_str(),
                ),
            db_type if db_type.contains("smallint") => {
                resp.push_str(db_type.replace("smallint", "SmallInt").as_str())
            }
            db_type if db_type.contains("mediumint") && db_type.contains("unsigned") => resp
                .push_str(
                    db_type
                        .replace("mediumint unsigned", "UnsignedMediumInt")
                        .as_str(),
                ),
            db_type if db_type.contains("mediumint") => {
                resp.push_str(db_type.replace("mediumint", "MediumInt").as_str())
            }
            db_type if db_type.contains("bigint") && db_type.contains("unsigned") => resp.push_str(
                db_type
                    .replace("bigint unsigned", "UnsignedBigInt")
                    .as_str(),
            ),
            db_type if db_type.contains("bigint") => {
                resp.push_str(db_type.replace("bigint", "BigInt").as_str())
            }
            db_type if db_type.contains("int") && db_type.contains("unsigned") => {
                resp.push_str(db_type.replace("int unsigned", "UnsignedInt").as_str())
            }
            db_type if db_type.contains("int") => {
                resp.push_str(db_type.replace("int", "Int").as_str())
            }
            db_type if db_type.contains("float") => resp.push_str("Float"),
            db_type if db_type.contains("double") => {
                resp.push_str(db_type.replace("double", "Double").as_str())
            }
            db_type if db_type.contains("decimal") => {
                resp.push_str(db_type.replace("decimal", "Decimal").as_str())
            }
            db_type if db_type.contains("datetime") => {
                resp.push_str(db_type.replace("datetime", "DateTime").as_str())
            }
            db_type if db_type.contains("date") => {
                resp.push_str(db_type.replace("date", "Date").as_str())
            }
            db_type if db_type.contains("time") => {
                resp.push_str(db_type.replace("time", "Time").as_str())
            }
            db_type if db_type.contains("timestamp") => {
                resp.push_str(db_type.replace("timestamp", "Timestamp").as_str())
            }
            db_type if db_type.contains("bool") => {
                resp.push_str(db_type.replace("boolean", "Boolean").as_str())
            }
            db_type if db_type.contains("varchar") => {
                resp.push_str(db_type.replace("varchar", "VarChar").as_str())
            }
            db_type if db_type.contains("mediumtext") => {
                resp.push_str(db_type.replace("mediumtext", "MediumText").as_str())
            }
            db_type if db_type.contains("text") => {
                resp.push_str(db_type.replace("text", "Text").as_str())
            }
            db_type if db_type.contains("json") => {
                resp.push_str(db_type.replace("json", "Json").as_str())
            }
            db_type if db_type.contains("Text") => {
                resp.push_str(db_type.replace("Text", "Text").as_str())
            }
            db_type if db_type.contains("Timestamp") => {
                resp.push_str(db_type.replace("Timestamp", "Timestamp").as_str())
            }
            db_type if db_type.contains("VarChar") => {
                resp.push_str(db_type.replace("VarChar", "VarChar").as_str())
            }
            db_type if db_type.contains("TinyInt") => {
                resp.push_str(db_type.replace("TinyInt", "TinyInt").as_str())
            }
            db_type if db_type.contains("SmallInt") => {
                resp.push_str(db_type.replace("SmallInt", "SmallInt").as_str())
            }
            db_type if db_type.contains("MediumInt") => {
                resp.push_str(db_type.replace("MediumInt", "MediumInt").as_str())
            }
            db_type if db_type.contains("BigInt") => {
                resp.push_str(db_type.replace("BigInt", "BigInt").as_str())
            }
            db_type if db_type.contains("Int") => {
                resp.push_str(db_type.replace("Int", "Int").as_str())
            }
            db_type if db_type.contains("Float") => {
                resp.push_str(db_type.replace("Float", "Float").as_str())
            }
            db_type if db_type.contains("Double") => {
                resp.push_str(db_type.replace("Double", "Double").as_str())
            }
            db_type if db_type.contains("Decimal") => {
                resp.push_str(db_type.replace("Decimal", "Decimal").as_str())
            }
            db_type if db_type.contains("DateTime") => {
                resp.push_str(db_type.replace("DateTime", "DateTime").as_str())
            }
            db_type if db_type.contains("Date") => {
                resp.push_str(db_type.replace("Date", "Date").as_str())
            }
            db_type if db_type.contains("Time") => {
                resp.push_str(db_type.replace("Time", "Time").as_str())
            }
            db_type if db_type.contains("Boolean") => {
                resp.push_str(db_type.replace("Boolean", "Boolean").as_str())
            }
            db_type if db_type.contains("Json") => {
                resp.push_str(db_type.replace("Json", "Json").as_str())
            }
            db_type if db_type.contains("Binary") => {
                resp.push_str(db_type.replace("Binary", "Binary").as_str())
            }
            db_type if db_type.contains("Blob") => {
                resp.push_str(db_type.replace("Blob", "Blob").as_str())
            }
            db_type if db_type.contains("MediumBlob") => {
                resp.push_str(db_type.replace("MediumBlob", "MediumBlob").as_str())
            }
            db_type if db_type.contains("LongBlob") => {
                resp.push_str(db_type.replace("LongBlob", "LongBlob").as_str())
            }
            db_type if db_type.contains("TinyBlob") => {
                resp.push_str(db_type.replace("TinyBlob", "TinyBlob").as_str())
            }

            _ => return None,
        }
        Some(resp)
    }
}

impl From<Description> for Field {
    fn from(description: Description) -> Self {
        let mut field = Field::new();
        field.set_name(description.field);
        field.set_is_id(description.key.contains("PRI"));
        field.set_is_required(description.null == "NO");
        field.set_unique(match description.key.contains("PRI") {
            true => Some(UniqueFlag { map: None }),
            false => match description.key.contains("UNI") {
                true => Some(UniqueFlag { map: None }),
                false => None,
            },
        });
        field.set_field_type(field.get_field_type_from_db_type());
        field.set_db_type_annotation(description.type_.clone());
        field.default = match description.default {
            Some(ref default) => match default.as_str() {
                "CURRENT_TIMESTAMP" => Some("now()".to_string()),
                "1" => match field.field_type.as_str() {
                    "Boolean" => Some("true".to_string()),
                    _ => Some(default.clone()),
                },
                "0" => match field.field_type.as_str() {
                    "Boolean" => Some("false".to_string()),
                    _ => Some(default.clone()),
                },
                _ => Some(default.clone()),
            },
            None => None,
        };
        field.set_target_name_length(field.name.len());
        field.set_field_type_length(field.field_type.len());
        field
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        let resp = self.name == other.name
            && self.field_type == other.field_type
            && self.is_id == other.is_id
            && self.is_required == other.is_required
            && self.unique == other.unique
            && self.default == other.default
            && self.db_type_annotation == other.db_type_annotation
            && self.relation == other.relation;
        resp
    }
}

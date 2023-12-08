use crate::remotes::sql::Table;
use crate::translators::prisma::field::Field;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
pub struct Model {
    name: String,
    fields: Vec<Field>,
    directives: Vec<String>,
    name_column_width: usize,
    field_type_column_width: usize,
}

impl Model {
    fn new() -> Model {
        Model {
            name: String::new(),
            fields: vec![],
            directives: vec![],
            name_column_width: 0,
            field_type_column_width: 0,
        }
    }

    pub fn as_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&format!("model {} {{\n", self.name));
        for field in &self.fields {
            text.push_str("  ");
            text.push_str(
                field
                    .as_text(
                        self.name_column_width,
                        self.field_type_column_width,
                        self.get_number_of_id_fields(),
                    )
                    .as_str(),
            );
            text.push_str("\n");
        }
        if self.directives.len() > 0 {
            for directive in &self.directives {
                text.push_str("\n");
                text.push_str(directive.as_str());
            }
            text.push_str("\n");
        }
        text.push_str("}\n");
        text
    }

    fn get_number_of_id_fields(&self) -> usize {
        let mut count = 0;
        for field in &self.fields {
            if field.is_id {
                count += 1;
            }
        }
        count
    }

    pub fn parse_from_disk(model_str: &str) -> Option<Model> {
        let mut name = String::new();
        let mut pieces: Vec<&str> = model_str.split("{").collect();
        let mut directives: Vec<String> = vec![];
        name.push_str(pieces.remove(0).replace(" ", "").as_str());
        if name.len() == 0 {
            return None;
        }
        assert!(pieces.len() == 1);
        let field_strs = pieces.pop().expect("pieces to have a value").split("\n");
        let mut fields: Vec<Field> = vec![];
        let mut name_column_width = 0;
        let mut field_type_column_width = 0;
        for field_str in field_strs {
            if field_str.len() == 0 {
                continue;
            }
            if field_str.trim().starts_with("@@") {
                directives.push(field_str.to_string());
                continue;
            }
            let field = Field::parse_from_disk(field_str.trim());
            if field.is_none() {
                continue;
            } else {
                if field.as_ref().unwrap().name.len() + 1 > name_column_width {
                    name_column_width = field.as_ref().unwrap().name.len() + 1;
                }
                if field.as_ref().unwrap().field_type.len() + 1 > field_type_column_width {
                    field_type_column_width = field.as_ref().unwrap().field_type.len() + 1;
                }
                fields.push(field.unwrap());
            }
        }

        for field in fields.iter_mut() {
            field.set_target_name_length(name_column_width);
            field.set_field_type_length(field_type_column_width);
        }

        Some(Model {
            name,
            fields,
            directives,
            name_column_width,
            field_type_column_width,
        })
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }
}

impl From<&Table> for Model {
    fn from(table: &Table) -> Self {
        let mut model = Model::new();
        model.set_name(table.name.clone());
        for description in table.description.clone().into_iter() {
            let field = Field::from(description);
            model.add_field(field);
        }
        if model.get_number_of_id_fields() > 1 {
            let number_of_directives = model.directives.len();
            model.directives.push(String::new());
            model.directives[number_of_directives].push_str("  @@id(["); // probably cursed to add
                                                                         // the whitespace like
                                                                         // this
            for field in model.fields.iter() {
                if field.is_id {
                    model.directives[number_of_directives]
                        .push_str(format!("{}, ", field.name).as_str());
                }
            }
            model.directives[number_of_directives] = model.directives[number_of_directives]
                .strip_suffix(", ")
                .unwrap()
                .to_string();
            model.directives[number_of_directives].push_str("])");
        }
        model.name_column_width = model.fields.iter().map(|f| f.name.len()).max().unwrap();
        model.field_type_column_width = model
            .fields
            .iter()
            .map(|f| f.field_type.len())
            .max()
            .unwrap();
        // todo: add directives
        model
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        if !self.name.eq(&other.name) {
            println!(
                "found mismatching name on {}. self: {}, other: {}",
                self.name, self.name, other.name
            );
            return false;
        }
        for directive in self.directives.iter() {
            let other_directive = other.directives.iter().find(|d| d == &directive);
            if other_directive.is_none() {
                println!("directive {} not found in other {}", directive, self.name);
                return false;
            }
        }
        for field in self.fields.iter() {
            println!("comparing field {} to other", field.name);
            let other_field = other.fields.iter().find(|f| f.name == field.name);
            if other_field.is_none() {
                println!("field {} not found in other", field.name);
                return false;
            }
            if field.to_owned() != other_field.expect("other field to exist").to_owned() {
                println!("field {} not equal to other", field.name);
                return false;
            }
        }
        if self.name_column_width != other.name_column_width {
            println!(
                "found mismatching name_column_width on {}. self: {}, other: {}",
                self.name, self.name_column_width, other.name_column_width
            );
            return false;
        }
        if self.field_type_column_width != other.field_type_column_width {
            println!(
                "found mismatching field_type_column_width on {}. self: {}, other: {}",
                self.name, self.field_type_column_width, other.field_type_column_width
            );
            return false;
        }
        true
    }
}

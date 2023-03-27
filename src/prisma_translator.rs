use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::sql::Table;
use crate::structure::TranslatorBehaviour;

pub struct PrismaTranslator<'a> {
    pub path: &'a Path,
}

impl PrismaTranslator<'_> {
    pub fn load(&self) {
        let file = File::open(self.path).unwrap();
        let mut generator_str = String::new();
        let mut data_source_str = String::new();
        let mut models_str = String::new();

        enum PushingTo {
            Generator,
            Datasource,
            Models,
        }

        let mut pushing_to: PushingTo = PushingTo::Generator;

        for line in BufReader::new(file).lines() {
            let current_line = line.unwrap();
            if current_line.starts_with("generator") {
                pushing_to = PushingTo::Generator;
            } else if current_line.starts_with("datasource") {
                pushing_to = PushingTo::Datasource;
            } else if current_line.starts_with("model") {
                pushing_to = PushingTo::Models;
            } else {
                pushing_to = pushing_to
            }
            match pushing_to {
                PushingTo::Generator => {
                    generator_str.push_str(&current_line);
                    generator_str.push_str("\n");
                }
                PushingTo::Datasource => {
                    data_source_str.push_str(&current_line);
                    data_source_str.push_str("\n");
                }
                PushingTo::Models => {
                    models_str.push_str(&current_line);
                    models_str.push_str("\n");
                }
            }
        }
        println!("Generator stuff:\n\n---------------\n");
        let generator = Generator::from_string(&generator_str);
        println!("Before {:?}", &generator_str);
        println!("After {:?}", generator.as_text());
        assert!(&generator_str == &generator.as_text());
        println!("{:?}", generator.as_text());
        println!(
            "Datasource stuff:\n\n---------------\n{:?}",
            data_source_str
        );
        println!("Models stuff:\n\n---------------\n{:?}", models_str);
    }
}

impl<'a> TranslatorBehaviour<serde_json::Value> for PrismaTranslator<'a> {
    fn from_database(&self, _database: &Vec<Table>) -> serde_json::Value {
        todo!();
    }
    fn from_disk(&self) -> Result<serde_json::Value, std::io::Error> {
        todo!("{:?}", self.load())
    }
    fn to_disk(&self, _database: &Vec<Table>) {
        todo!();
    }
}

pub struct PrismaSchema {
    generator: Generator,
    datasource: Datasource,
    models: Vec<Model>,
}

impl PrismaSchema {
    pub fn new() -> PrismaSchema {
        PrismaSchema {
            generator: Generator::new(),
            datasource: Datasource::new(),
            models: vec![],
        }
    }
    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }
    pub fn as_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&self.generator.as_text());
        text.push_str(&self.datasource.as_text());
        for model in &self.models {
            text.push_str(&model.as_text());
        }
        text
    }
    pub fn load(&self, full_path: &String) -> String {
        let file = File::open(full_path).unwrap();
        let mut contents = String::new();
        for line in BufReader::new(file).lines() {
            contents.push_str(&line.unwrap());
            contents.push_str("\n");
        }
        println!("{:?}", contents);
        contents
    }
    pub fn write(&self, full_path: &String) -> Result<(), std::io::Error> {
        let contents = self.as_text();
        fs::write(full_path, contents)?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Generator {
    name: String,
    provider: String,
}

impl Generator {
    fn new() -> Generator {
        Generator {
            name: String::from("client"),
            provider: String::from("prisma-client-js"),
        }
    }
    fn from_string(generator_str: &String) -> Generator {
        let pieces: Vec<&str> = generator_str.split(" = ").collect();
        let provider_piece = pieces[1];
        let provider_pieces: Vec<&str> = provider_piece.split(" ").collect();
        let binding = String::from(provider_pieces[0].replace('"', "").replace("\n", ""));
        let provider_dirty: Vec<&str> = binding.split("}").collect();
        let provider = String::from(provider_dirty[0]);
        println!("{:?}", provider);
        Generator {
            name: String::from("client"),
            provider,
        }
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_provider(&mut self, provider: String) {
        self.provider = provider;
    }
    fn as_text(&self) -> String {
        format!(
            "generator {0} {{\n  provider = \"{1}\"\n}}\n\n",
            self.name, self.provider
        )
    }
}

#[derive(Deserialize, Serialize)]
struct Datasource {
    name: String,
    provider: String,
}

impl Datasource {
    fn new() -> Datasource {
        Datasource {
            name: String::from("db"),
            provider: String::from("mysql"),
        }
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_provider(&mut self, provider: String) {
        self.provider = provider;
    }
    fn as_text(&self) -> String {
        "datasource {} {\n  provider = \"{}\"\n  url = env(\"DATABASE_URL\")".to_string()
    }
}

#[derive(Deserialize, Serialize)]
pub struct Model {
    name: String,
    fields: Vec<Field>,
}

impl Model {
    fn new() -> Model {
        Model {
            name: String::new(),
            fields: vec![],
        }
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }
    fn as_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&format!("model {} {{\n", self.name));
        for field in &self.fields {
            text.push_str(&format!("  {} {}\n", field.name, field.field_type));
        }
        text.push_str("}\n");
        text
    }
}

#[derive(Deserialize, Serialize)]
struct Field {
    name: String,
    field_type: String,
    is_required: bool,
    is_list: bool,
    is_unique: bool,
    is_id: bool,
    is_read_only: bool,
    is_generated: bool,
    is_relation: bool,
    relation_name: String,
    relation_from: String,
    relation_to: String,
    relation_on_delete: String,
    relation_on_update: String,
}

impl Field {
    fn new() -> Field {
        Field {
            name: String::new(),
            field_type: String::new(),
            is_required: false,
            is_list: false,
            is_unique: false,
            is_id: false,
            is_read_only: false,
            is_generated: false,
            is_relation: false,
            relation_name: String::new(),
            relation_from: String::new(),
            relation_to: String::new(),
            relation_on_delete: String::new(),
            relation_on_update: String::new(),
        }
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_field_type(&mut self, field_type: String) {
        self.field_type = field_type;
    }
    fn set_is_required(&mut self, is_required: bool) {
        self.is_required = is_required;
    }
    fn set_is_list(&mut self, is_list: bool) {
        self.is_list = is_list;
    }
    fn set_is_unique(&mut self, is_unique: bool) {
        self.is_unique = is_unique;
    }
    fn set_is_id(&mut self, is_id: bool) {
        self.is_id = is_id;
    }
    fn set_is_read_only(&mut self, is_read_only: bool) {
        self.is_read_only = is_read_only;
    }
    fn set_is_generated(&mut self, is_generated: bool) {
        self.is_generated = is_generated;
    }
    fn set_is_relation(&mut self, is_relation: bool) {
        self.is_relation = is_relation;
    }
    fn set_relation_name(&mut self, relation_name: String) {
        self.relation_name = relation_name;
    }
    fn set_relation_from(&mut self, relation_from: String) {
        self.relation_from = relation_from;
    }
    fn set_relation_to(&mut self, relation_to: String) {
        self.relation_to = relation_to;
    }
    fn set_relation_on_delete(&mut self, relation_on_delete: String) {
        self.relation_on_delete = relation_on_delete;
    }
    fn set_relation_on_update(&mut self, relation_on_update: String) {
        self.relation_on_update = relation_on_update;
    }
    fn as_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&self.name);
        text.push_str("        ");
        text.push_str(&self.field_type);
        if self.is_list {
            text.push_str("[]");
        }
        if !self.is_required {
            text.push_str("?");
        }
        text.push_str("        ");
        text.push_str(&self.field_type);
        if self.is_unique {
            text.push_str("@unique()");
        }
        if self.is_id {
            text.push_str("@id");
        }
        if self.is_read_only {
            text.push_str("@readonly");
        }
        if self.is_generated {
            text.push_str("@default");
        }
        if self.is_relation {
            text.push_str("@relation");
        }
        text
    }
}

use crate::remotes::sql::Table;
use crate::translators::behaviour::TranslatorBehaviour;
use crate::translators::prisma::{
    data_source::Datasource, generator::Generator, model::Model, schema::PrismaSchema,
};
use anyhow::Result;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub struct PrismaTranslator {
    pub path: String,
    pub disk_schema: Option<PrismaSchema>,
    pub db_schema: Option<PrismaSchema>,
}

impl TranslatorBehaviour<PrismaSchema> for PrismaTranslator {
    fn get_translation(&self, database: &Vec<Table>) -> PrismaSchema {
        PrismaSchema::from(database)
    }

    fn load_from_database(&mut self, database: &Vec<Table>) {
        self.db_schema = Some(self.get_translation(database));
    }

    fn load_from_disk(&mut self) -> Result<()> {
        let loaded = self.parse_from_disk()?;
        if self.disk_schema.is_none() {
            self.disk_schema = Some(loaded);
        }
        Ok(())
    }

    fn write_to_disk(&self, database: &Vec<Table>) {
        let mut output_path_buf = PathBuf::new();
        output_path_buf.push(&self.path);
        let output_path_buf = output_path_buf.with_file_name("mysql_output_from_db.prisma");
        let output_path = output_path_buf.as_path();
        let schema = PrismaSchema::from(database);
        let res = fs::write(output_path.to_str().unwrap(), schema.as_text());
        if res.is_ok() {
            println!("success");
        } else {
            println!("failed");
        }
    }

    fn get_string(&self) -> String {
        if self.disk_schema.is_none() && self.db_schema.is_none() {
            return String::new();
        }
        let mut resp = String::new();
        if self.disk_schema.is_some() {
            resp.push_str("disk schema:\n\n");
            resp.push_str(&self.disk_schema.as_ref().unwrap().as_text());
            resp.push_str("\n\n");
        }
        if self.db_schema.is_some() {
            resp.push_str("db schema:\n\n");
            resp.push_str(&self.db_schema.as_ref().unwrap().as_text());
            resp.push_str("\n\n");
        }
        resp
    }
}

impl PrismaTranslator {
    pub fn parse_from_disk(&self) -> Result<PrismaSchema> {
        let file = File::open(self.path.clone()).unwrap();
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
        let generator = Generator::parse_from_disk(&generator_str);
        let datasource = Datasource::parse_from_disk(&data_source_str);
        let mut models: Vec<Model> = vec![];
        let models_strs: Vec<&str> = models_str.split("model ").collect();
        for curr_model_str in models_strs {
            let model = Model::parse_from_disk(curr_model_str);
            if model.is_none() {
                continue;
            }
            models.push(model.unwrap());
        }
        Ok(PrismaSchema::build(generator, datasource, models))
    }
}

use crate::remotes::sql::Table;
use crate::translators::prisma::{data_source::Datasource, generator::Generator, model::Model};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
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

    pub fn build(generator: Generator, datasource: Datasource, models: Vec<Model>) -> PrismaSchema {
        PrismaSchema {
            generator,
            datasource,
            models,
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn as_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&self.generator.as_text());
        text.push_str("\n\n");
        text.push_str(&self.datasource.as_text());
        text.push_str("\n\n");
        for model in &self.models {
            text.push_str(&model.as_text());
            text.push_str("\n");
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
        contents
    }

    pub fn write(&self, full_path: &String) -> Result<()> {
        let contents = self.as_text();
        fs::write(full_path, contents)?;
        Ok(())
    }
}

impl PartialEq for PrismaSchema {
    fn eq(&self, other: &Self) -> bool {
        self.generator == other.generator
            && self.datasource == other.datasource
            && self.models == other.models
    }
}

impl From<&Vec<Table>> for PrismaSchema {
    fn from(tables: &Vec<Table>) -> Self {
        let mut prisma_schema = PrismaSchema::new();
        for table in tables {
            let model = Model::from(table);
            prisma_schema.add_model(model);
        }
        prisma_schema
    }
}

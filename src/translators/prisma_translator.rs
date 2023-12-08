use crate::remotes::sql::{Description, Table};
use crate::translators::behaviour::TranslatorBehaviour;
use anyhow::Result;
use serde::{Deserialize, Serialize};
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

    fn build(generator: Generator, datasource: Datasource, models: Vec<Model>) -> PrismaSchema {
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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
    fn parse_from_disk(generator_str: &String) -> Generator {
        let pieces: Vec<&str> = generator_str.split(" = ").collect();
        let provider_piece = pieces[1];
        let provider_pieces: Vec<&str> = provider_piece.split(" ").collect();
        let binding = String::from(provider_pieces[0].replace('"', "").replace("\n", ""));
        let provider_dirty: Vec<&str> = binding.split("}").collect();
        let provider = String::from(provider_dirty[0]);
        Generator {
            name: String::from("client"),
            provider,
        }
    }
    fn as_text(&self) -> String {
        format!(
            "generator {0} {{\n  provider = \"{1}\"\n}}",
            self.name, self.provider
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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
    fn as_text(&self) -> String {
        format!(
            "datasource {} {{\n  provider = \"{}\"\n  url      = env(\"DATABASE_URL\")\n}}",
            self.name, self.provider
        )
    }
    fn parse_from_disk(datasource_string: &String) -> Datasource {
        let lines: Vec<&str> = datasource_string.split("\n").collect();
        let mut name = String::new();
        let mut provider = String::new();
        for line in lines {
            if line.contains("datasource") {
                name.push_str(
                    line.strip_prefix("datasource ")
                        .unwrap()
                        .strip_suffix(" {")
                        .unwrap(),
                )
            }
            if line.contains("provider") {
                let pieces: Vec<&str> = line.split("=").collect();
                provider = pieces[1].replace('"', "").replace(' ', "");
            }
        }
        Datasource { name, provider }
    }
}

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

    fn as_text(&self) -> String {
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

    fn parse_from_disk(model_str: &str) -> Option<Model> {
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

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
struct Relation {
    map: Option<String>,
    fields: Option<Vec<String>>,
    references: Option<Vec<String>>,
    on_update: Option<String>,
    on_delete: Option<String>,
}

impl Relation {
    fn as_text(&self) -> String {
        let mut resp = String::new();
        resp.push_str("@relation(");
        if self.fields.is_some() {
            resp.push_str("fields: [");
            resp.push_str(self.fields.clone().unwrap().join(", ").as_str());
            resp.push_str("]");
        }
        if self.references.is_some() {
            if self.fields.is_some() {
                resp.push_str(", ");
            }
            resp.push_str("references: [");
            resp.push_str(self.references.clone().unwrap().join(", ").as_str());
            resp.push_str("]");
        }
        if self.map.is_some() {
            if self.references.is_some() {
                resp.push_str(", ");
            }
            resp.push_str("map: \"");
            resp.push_str(self.map.clone().unwrap().as_str());
            resp.push_str("\"");
        }
        if self.on_delete.is_some() {
            resp.push_str(", ");
            resp.push_str("onDelete: ");
            resp.push_str(self.on_delete.clone().unwrap().as_str());
        }
        if self.on_update.is_some() {
            resp.push_str(", ");
            resp.push_str("onUpdate: ");
            resp.push_str(self.on_update.clone().unwrap().as_str());
        }
        resp.push_str(")");
        resp
    }

    fn from_string(string: String) -> Relation {
        let mut this_fields: Vec<String> = vec![];
        let mut this_references: Vec<String> = vec![];
        let mut this_map: String = String::new();
        let mut this_on_update: String = String::new();
        let mut this_on_delete: String = String::new();

        println!("---> constructing relation from {}", string);

        let mut new_string = string.clone();

        new_string = new_string
            .strip_prefix("@relation(")
            .expect("input to be a relation string")
            .to_string();

        new_string = new_string
            .strip_suffix(")")
            .expect("relation string to have a closing paren")
            .to_string();

        let pieces = new_string.split(", ");

        println!("---> pieces");
        for piece in pieces {
            if piece.starts_with("fields:") {
                println!("{:?}", piece);
                let new_piece = piece
                    .strip_prefix("fields:[")
                    .expect("fields to be formatted correctly")
                    .strip_suffix("]")
                    .expect("fields to be formatted correctly");
                let new_pieces = Some(new_piece.split(", "));
                for sub_piece in new_pieces.unwrap() {
                    this_fields.push(String::from(sub_piece.to_string()));
                }
            } else if piece.starts_with("references:") {
                let new_piece = piece
                    .strip_prefix("references:[")
                    .expect("references to be formatted correctly")
                    .strip_suffix("]")
                    .expect("references to be formatted correctly");
                let new_pieces = Some(new_piece.split(", "));
                for sub_piece in new_pieces.unwrap() {
                    this_references.push(String::from(sub_piece.to_string()));
                }
            } else if piece.starts_with("onDelete:") {
                this_on_delete = piece
                    .strip_prefix("onDelete:")
                    .expect("on delete to be formatted correctly")
                    .to_string();
            } else if piece.starts_with("onUpdate:") {
                this_on_update = piece
                    .strip_prefix("onUpdate:")
                    .expect("on delete to be formatted correctly")
                    .to_string();
            } else if piece.starts_with("map:") {
                this_map = piece
                    .strip_prefix("map:\"")
                    .expect("map to be formatted correctly")
                    .strip_suffix("\"")
                    .expect("closing double quotes to exist")
                    .to_string();
            } else {
                panic!("unhandled relation {:?}", piece);
            }
        }

        let fields = if this_fields.len() > 0 {
            Some(this_fields)
        } else {
            None
        };

        let references = if this_references.len() > 0 {
            Some(this_references)
        } else {
            None
        };

        let map = if this_map.len() > 0 {
            Some(this_map)
        } else {
            None
        };

        let on_update = if this_on_update.len() > 0 {
            Some(this_on_update)
        } else {
            None
        };

        let on_delete = if this_on_delete.len() > 0 {
            Some(this_on_delete)
        } else {
            None
        };

        Relation {
            fields,
            references,
            map,
            on_update,
            on_delete,
        }
    }
}

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        let resp = self.fields == other.fields
            && self.references == other.references
            && self.map == other.map
            && self.on_update == other.on_update
            && self.on_delete == other.on_delete;
        if !resp {
            println!("in relation");
            println!("---> {:?} != {:?}", self.fields, other.fields);
            println!("---> {:?} != {:?}", self.references, other.references);
            println!("---> {:?} != {:?}", self.map, other.map);
            println!("---> {:?} != {:?}", self.on_update, other.on_update);
            println!("---> {:?} != {:?}", self.on_delete, other.on_delete);
        }
        resp
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
struct UniqueFlag {
    pub map: Option<String>,
}

impl UniqueFlag {
    pub fn as_text(self) -> String {
        let mut resp = String::new();
        resp.push_str("@unique");
        if self.map.is_some() {
            resp.push_str("(map: \"");
            resp.push_str(self.map.clone().unwrap().as_str());
            resp.push_str("\")");
        }
        resp
    }
}

impl PartialEq for UniqueFlag {
    fn eq(&self, other: &Self) -> bool {
        let resp = self.map == other.map;
        if !resp {
            println!("in unique flag");
            println!("---> {:?} != {:?}", self.map, other.map);
        }
        self.map == other.map
    }
}

impl From<&str> for UniqueFlag {
    fn from(string: &str) -> Self {
        let mut new_string = string.clone();

        if new_string.starts_with("@") {
            new_string = new_string
                .strip_prefix("@")
                .expect("input to be a unique string");
        }

        new_string = new_string
            .strip_prefix("unique")
            .expect("input to be a unique string");

        if new_string.len() == 0 {
            return UniqueFlag { map: None };
        }

        new_string = new_string
            .trim()
            .strip_prefix("(")
            .expect("unique string to start with an opening paren")
            .strip_suffix(")")
            .expect("unique string to end in a closing paren");

        new_string = new_string
            .strip_prefix("map: \"")
            .expect("map to be formatted correctly")
            .strip_suffix("\"")
            .expect("closing double quotes to exist");

        UniqueFlag {
            map: Some(new_string.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
struct Field {
    name: String,
    target_name_width: usize,
    field_type: String,
    target_field_type_width: usize,
    db_type_annotation: Option<String>,
    is_required: bool,
    is_array: bool,
    is_id: bool,
    relation: Option<Relation>,
    default: Option<String>,
    unique: Option<UniqueFlag>,
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
    fn parse_from_disk(field_str: &str) -> Option<Field> {
        if field_str.starts_with("@@") || field_str.starts_with("@relation") {
            panic!("uncaught directive {}", field_str);
        }
        let mut field_str_mut = field_str.clone().to_string();
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
    fn set_target_name_length(&mut self, length: usize) {
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
    fn set_field_type_length(&mut self, length: usize) {
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
    // fn set_relation(&mut self, relation: Option<Relation>) {
    //     self.relation = relation;
    // }
    fn as_text(
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
    fn get_field_type_from_db_type(&self, db_type: &str) -> String {
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
            _ => return None,
        }
        Some(resp)
    }
}

impl From<Description> for Field {
    fn from(description: Description) -> Self {
        println!("description: {:?}", description);
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
        field.set_field_type(field.get_field_type_from_db_type(description.type_.as_str()));
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
        // println!("field: {:?}", field);
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
        if !resp {
            println!("self: {:?}", self);
            println!("other: {:?}", other);
        }
        resp
    }
}

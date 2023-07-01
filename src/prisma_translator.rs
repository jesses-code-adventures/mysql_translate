use crate::sql::Table;
use crate::structure::TranslatorBehaviour;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct PrismaTranslator<'a> {
    pub path: &'a Path,
    pub prisma_schema: Option<PrismaSchema>,
}

impl<'a> TranslatorBehaviour<PrismaSchema> for PrismaTranslator<'a> {
    fn from_database(&self, _database: &Vec<Table>) -> PrismaSchema {
        todo!();
        // for table in database {
        //     println!("{:?}", table);
        // }
        // PrismaSchema::new()
    }

    fn from_disk(&mut self) -> Result<PrismaSchema> {
        if self.prisma_schema.is_none() {
            self.prisma_schema = Some(self.parse_from_disk());
        }
        Ok(self.prisma_schema.clone().unwrap())
    }

    fn to_disk(&self, _database: &Vec<Table>) {
        todo!();
    }

    fn display(&self) {
        if self.prisma_schema.is_none() {
            return;
        }
        println!("{}", self.prisma_schema.as_ref().unwrap().as_text());
        let output_path = self.path.with_file_name("mysql_output.prisma");
        println!("writing file to {}", output_path.display());
        let res = fs::write(output_path, self.prisma_schema.as_ref().unwrap().as_text());
        if res.is_ok() {
            println!("success");
        } else {
            println!("failed");
        }
    }
}

impl PrismaTranslator<'_> {
    pub fn parse_from_disk(&self) -> PrismaSchema {
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
        PrismaSchema::build(generator, datasource, models)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_provider(&mut self, provider: String) {
        self.provider = provider;
    }
    fn as_text(&self) -> String {
        format!(
            "generator {0} {{\n  provider = \"{1}\"\n}}",
            self.name, self.provider
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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

#[derive(Deserialize, Serialize, Debug, Clone)]
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
            text.push_str(field.as_text().as_str());
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

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Relation {
    map: Option<String>,
    fields: Option<Vec<String>>,
    references: Option<Vec<String>>,
    on_update: Option<String>,
    on_delete: Option<String>,
}

impl Relation {
    fn new(
        map: Option<String>,
        fields: Option<Vec<String>>,
        references: Option<Vec<String>>,
        on_update: Option<String>,
        on_delete: Option<String>,
    ) -> Relation {
        let relation = Relation {
            map,
            fields,
            references,
            on_update,
            on_delete,
        };
        if relation.map.is_none() || (relation.fields.is_none() && relation.references.is_none()) {
            panic!("insufficient data found to construct relation");
        }
        relation
    }

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
            .expect("relation string to end in a closing paren")
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

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Field {
    name: String,
    target_name_width: usize,
    field_type: String,
    target_field_type_width: usize,
    db_type_annotation: Option<String>,
    is_required: bool,
    is_array: bool,
    is_unique: bool,
    is_id: bool,
    is_read_only: bool,
    relation: Option<Relation>,
    default: Option<String>,
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
            is_unique: false,
            is_id: false,
            is_read_only: false,
            relation: None,
            default: None,
        }
    }
    fn parse_from_disk(field_str: &str) -> Option<Field> {
        if field_str.starts_with("@@") || field_str.starts_with("@relation") {
            panic!("uncaught directive {}", field_str);
        }
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
        let mut is_unique = false;
        let mut is_required = true;
        let mut is_id = false;
        let mut default: Option<String> = None;
        let pieces: Vec<&str> = field_str.split(" ").collect();
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
            if piece.contains("@unique") {
                is_unique = true;
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
            println!("Unterminated delimiter in {}", field_str);
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
            is_read_only: false,
            is_unique,
            relation,
            default,
        };
        Some(field)
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_target_name_length(&mut self, length: usize) {
        self.target_name_width = length;
    }
    fn set_field_type(&mut self, field_type: String) {
        self.field_type = field_type;
    }
    fn set_field_type_length(&mut self, length: usize) {
        self.target_field_type_width = length;
    }
    fn set_is_required(&mut self, is_required: bool) {
        self.is_required = is_required;
    }
    fn set_is_list(&mut self, is_array: bool) {
        self.is_array = is_array;
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
    fn set_relation(&mut self, relation: Option<Relation>) {
        self.relation = relation;
    }
    fn as_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&self.name);
        let spaces_to_add = self.target_name_width - self.name.len();
        for _ in 0..spaces_to_add {
            text.push_str(" ");
        }
        text.push_str(&self.field_type);
        let spaces_to_add = self.target_field_type_width - self.field_type.len();
        for _ in 0..spaces_to_add {
            text.push_str(" ");
        }
        if self.is_id {
            text.push_str("@id ");
        }
        if self.is_unique {
            text.push_str("@unique() ");
        }
        if self.default.is_some() {
            text.push_str("@default(");
            text.push_str(self.default.as_ref().unwrap().as_str());
            text.push_str(") ");
        }
        if self.db_type_annotation.is_some() {
            text.push_str(self.db_type_annotation.clone().unwrap().as_str());
            text.push_str(" ");
        }
        if self.is_read_only {
            text.push_str("@readonly ");
        }
        if self.relation.is_some() {
            text.push_str(self.relation.clone().unwrap().as_text().as_str());
        }
        text
    }
}

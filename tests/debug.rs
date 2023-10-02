use mysql_translate::{
    session::Session,
    set_env_variables,
    structure::{AcceptedFormat, DiskMapping, TranslatorBehaviour},
    translators::{json_translator::JsonTranslator, prisma_translator::PrismaTranslator},
};
use std::env;

#[test]
pub fn prisma_db_pull() {
    set_env_variables::set_vars();
    let mut data_location =
        env::var("STORAGE").expect("storage directory to exist as an environment variable");
    data_location.push_str("/session.json");
    let session = Session::new(&data_location.to_string()).expect("session to load");
    let mut prisma_disk_mappings: Vec<&DiskMapping> = session.databases[0]
        .disk_mappings
        .iter()
        .filter(|x| x.format == AcceptedFormat::Prisma)
        .collect::<Vec<&DiskMapping>>();
    let prisma_disk_mapping = prisma_disk_mappings
        .pop()
        .expect("prisma disk mapping to exist");
    let mut translator = PrismaTranslator {
        path: prisma_disk_mapping.path.to_owned(),
        disk_schema: None,
        db_schema: None,
    };
    let descriptions = session.databases[0].get_descriptions();
    translator.load_from_database(&descriptions);
    assert!(!translator.db_schema.is_none());
}

#[test]
pub fn json_db_pull() {
    set_env_variables::set_vars();
    let mut data_location =
        env::var("STORAGE").expect("storage directory to exist as an environment variable");
    data_location.push_str("/session.json");
    let session = Session::new(&data_location.to_string()).expect("session to load");
    let mut prisma_disk_mappings: Vec<&DiskMapping> = session.databases[0]
        .disk_mappings
        .iter()
        .filter(|x| x.format == AcceptedFormat::Json)
        .collect::<Vec<&DiskMapping>>();
    let prisma_disk_mapping = prisma_disk_mappings
        .pop()
        .expect("prisma disk mapping to exist");
    let mut translator = JsonTranslator {
        path: prisma_disk_mapping.path.to_owned(),
        json: None,
    };
    let descriptions = session.databases[0].get_descriptions();
    translator.load_from_database(&descriptions);
    assert!(!translator.json.is_none());
}

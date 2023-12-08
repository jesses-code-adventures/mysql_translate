use mysql_translate::{
    functionality::{
        session::Session,
        structure::{get_session_data_location, AcceptedFormat, DiskMapping},
    },
    translators::{
        behaviour::TranslatorBehaviour, json_translator::JsonTranslator,
        prisma_translator::PrismaTranslator,
    },
};

#[test]
pub fn prisma_db_write() {
    let session_data_location = get_session_data_location();
    let session = Session::new(&session_data_location.to_string()).expect("session to load");
    let mut prisma_disk_mappings: Vec<&DiskMapping> = session.databases[0]
        .disk_mappings
        .iter()
        .filter(|x| x.format == AcceptedFormat::Prisma)
        .collect::<Vec<&DiskMapping>>();
    let prisma_disk_mapping = prisma_disk_mappings
        .pop()
        .expect("prisma disk mapping to exist");
    let translator = PrismaTranslator {
        path: prisma_disk_mapping.path.to_owned(),
        disk_schema: None,
        db_schema: None,
    };
    let descriptions = session.databases[0].get_descriptions();
    translator.write_to_disk(&descriptions)
}

#[test]
pub fn json_db_pull() {
    let session_data_location = get_session_data_location();
    let session = Session::new(&session_data_location.to_string()).expect("session to load");
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

fn main() {
    prisma_db_write();
}

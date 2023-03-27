use crate::sql::Table;
use core::fmt::{self, Display};
use serde::{Deserialize, Serialize};
use serde_json;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// A trait for translating between a database of Vec<Table> and the
/// format implemented by the child struct.
pub trait TranslatorBehaviour<T> {
    /// Writes the database output straight to the disk in the desired format.
    fn to_disk(&self, descriptions: &Vec<Table>);
    /// Converts a vector of sql::Table to a json object.
    /// May be changed to a different format in the future - this should
    /// be the shared intemediary format between formats, so to speak.
    fn from_database(&self, descriptions: &Vec<Table>) -> T;
    /// Reads the database description from disk in the desired format.
    fn from_disk(&self) -> Result<T, std::io::Error>;
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct DiskMapping<'a> {
    pub format: AcceptedFormat,
    #[serde(borrow)]
    pub path: Cow<'a, Path>,
}

impl<'a> DiskMapping<'a> {
    pub fn from_json(
        json: serde_json::Value,
    ) -> Result<Vec<DiskMapping<'static>>, serde_json::Error> {
        // Temporary deserialization struct
        #[derive(Deserialize)]
        struct TempMapping {
            format: AcceptedFormat,
            path: String,
        }

        let temp_mappings: Vec<TempMapping> = serde_json::from_value(json)?;
        let disk_mappings: Vec<DiskMapping<'_>> = temp_mappings
            .into_iter()
            .map(|temp_mapping| {
                let path_buf = PathBuf::from(temp_mapping.path);
                let path: Cow<'_, Path> = Cow::Owned(path_buf);
                DiskMapping {
                    format: temp_mapping.format,
                    path,
                }
            })
            .collect();

        Ok(disk_mappings)
    }
}

#[derive(Serialize, PartialEq, Deserialize, Copy, Clone, Debug)]
pub enum AcceptedFormat {
    Json,
    Prisma,
}

impl AcceptedFormat {
    pub fn from_string(format: &str) -> AcceptedFormat {
        match format {
            "json" => AcceptedFormat::Json,
            "prisma" => AcceptedFormat::Prisma,
            _ => panic!("Invalid format"),
        }
    }
    pub fn as_string(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Prisma => "prisma",
        }
    }
    pub fn all_as_array() -> Vec<AcceptedFormat> {
        vec![AcceptedFormat::Json, AcceptedFormat::Prisma]
    }
}

impl Display for AcceptedFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_string())
    }
}

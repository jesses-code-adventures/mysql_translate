use anyhow::Result;
use core::fmt::{self, Display};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;

pub fn set_vars() {
    dotenv().expect(".env not found");
}

pub fn get_session_data_location() -> String {
    set_vars();
    let mut data_location =
        env::var("STORAGE").expect("storage directory to exist as an environment variable");
    data_location.push_str("/session.json");
    data_location
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct DiskMapping {
    pub format: AcceptedFormat,
    // #[serde(borrow)]
    pub path: String,
}

impl DiskMapping {
    pub fn from_json(json: serde_json::Value) -> Result<Vec<DiskMapping>, serde_json::Error> {
        // Temporary deserialization struct
        #[derive(Deserialize)]
        struct TempMapping {
            format: AcceptedFormat,
            path: String,
        }

        let temp_mappings: Vec<TempMapping> = serde_json::from_value(json)?;
        let disk_mappings: Vec<DiskMapping> = temp_mappings
            .into_iter()
            .map(|temp_mapping| DiskMapping {
                format: temp_mapping.format,
                path: temp_mapping.path,
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
    pub fn from_string(format: &str) -> Option<AcceptedFormat> {
        match format {
            "json" => Some(AcceptedFormat::Json),
            "prisma" => Some(AcceptedFormat::Prisma),
            _ => None,
        }
    }
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Prisma => "prisma",
        }
    }
    pub fn all_as_array() -> Vec<AcceptedFormat> {
        vec![AcceptedFormat::Json, AcceptedFormat::Prisma]
    }
    pub fn all_as_string_array() -> Vec<String> {
        AcceptedFormat::all_as_array()
            .into_iter()
            .map(|format| format.as_string().to_string())
            .collect()
    }
    pub fn all_as_str_array() -> Vec<&'static str> {
        AcceptedFormat::all_as_array()
            .into_iter()
            .map(|format| format.as_string())
            .collect()
    }
}

impl Display for AcceptedFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_string())
    }
}

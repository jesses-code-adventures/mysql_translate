use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Generator {
    name: String,
    provider: String,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            name: String::from("client"),
            provider: String::from("prisma-client-js"),
        }
    }
    pub fn parse_from_disk(generator_str: &String) -> Generator {
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
    pub fn as_text(&self) -> String {
        format!(
            "generator {0} {{\n  provider = \"{1}\"\n}}",
            self.name, self.provider
        )
    }
}

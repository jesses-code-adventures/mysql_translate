use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
pub struct Relation {
    map: Option<String>,
    fields: Option<Vec<String>>,
    references: Option<Vec<String>>,
    on_update: Option<String>,
    on_delete: Option<String>,
}

impl Relation {
    pub fn as_text(&self) -> String {
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

    pub fn from_string(string: String) -> Relation {
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

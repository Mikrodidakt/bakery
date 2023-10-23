pub mod settings;
pub mod context;

pub use settings::WsSettings;
pub use context::Context;

use indexmap::IndexMap;
use serde_json::Value;
use crate::error::BError;
pub trait Config {
    fn get_str_manifest(name: &str, data: &Value, default: Option<String>) -> Result<String, BError> {
        match data.get(name) {
            Some(value) => {
                if value.is_object() {
                    return Ok(value.to_string());
                }
                return Err(BError::ParseError(format!("Failed to parse manifest. Error when reading object '{}'", name)));
            }
            None => {
                match default {
                    Some(default_value) => Ok(default_value),
                    None => Err(BError::ValueError(format!("Failed to read manifest value '{}'", name))),
                }
            }
        }
    }

    fn get_str_value(name: &str, data: &Value, default: Option<String>) -> Result<String, BError> {
        let value: Option<&str> = data.get(name).and_then(|v| v.as_str());
        match value {
            Some(value) => {
                return Ok(value.to_string());
            },
            None => {
                match default {
                    Some(default_value) => Ok(default_value),
                    None => Err(BError::ValueError(format!("Failed to read string value '{}'", name))),
                }
            }
        }
    }

    fn get_array_value(name: &str, data: &Value, default: Option<Vec<String>>) -> Result<Vec<String>, BError> {
        match data.get(name) {
            Some(array_value) => {
                if array_value.is_array() {
                    return Ok(array_value
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_owned())
                    .collect());
                } else {
                    return Err(BError::ParseError(format!("Failed to read array '{}'", name)));
                }
            }
            None => {
                match default {
                    Some(default_value) => Ok(default_value),
                    None => Err(BError::ValueError(format!("Failed to read array value '{}'", name))),
                }
            }
        }
    }

    fn get_hashmap_value(name: &str, data: &Value) -> Result<IndexMap<String, String>, BError> {
        match data.get(name) {
            Some(array_value) => {
                if array_value.is_array() {
                    let mut hashmap: IndexMap<String, String> = IndexMap::new();
                    for value in array_value.as_array().unwrap().iter() {
                        let pair: String = value.to_string();
                        let parts: Vec<&str> = pair.splitn(2, '=').collect();
                        
                        if parts.len() == 2 {
                            let key = parts[0].to_string();
                            let value = parts[1].to_string();
                            hashmap.insert(
                                String::from(key.trim_matches('"')),
                                String::from(value.trim_matches('"'))
                            );
                        }
                    }
                    Ok(hashmap)
                } else {
                    return Err(BError::ParseError(format!("Failed to parse hashmap. Error when reading object '{}'", name)));
                }
            }
            None => {
                return Ok(IndexMap::new());
            }
        }
    }

    fn get_value<'a>(name: &str, data: &'a Value) -> Result<&'a Value, BError> {
        match data.get(name) {
            Some(value) => Ok(value),
            None => Err(BError::ParseError(format!("Failed to get value '{}'", name))),
        }
    }

    fn parse(json_string: &str) -> Result<Value, BError> {
        match serde_json::from_str(json_string) {
            Ok(data) => {
                Ok(data) 
            },
            Err(err) => Err(BError::ParseError(format!("Failed to parse JSON: {}", err))),
        }
    }
}
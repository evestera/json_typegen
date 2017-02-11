pub use serde_json::value::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Spec {
    pub swagger: String,
    pub info: Info,
    pub host: Option<String>,
    #[serde(rename = "basePath")]
    pub base_path: Option<String>,
    pub paths: HashMap<String, Value>,
    pub definitions: Option<HashMap<String, RefOr<SchemaObject>>>,
    pub parameters: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PathItem {
    #[serde(rename = "$ref")]
    pub ref_path: Option<String>,
    pub get: Option<Operation>,
    pub put: Option<Operation>,
    pub post: Option<Operation>,
    pub delete: Option<Operation>,
    pub options: Option<Operation>,
    pub head: Option<Operation>,
    pub patch: Option<Operation>,
    #[serde(default)]
    pub parameters: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Operation {}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RefOr<T> {
    Ref {
        #[serde(rename = "$ref")]
        path: String
    },
    Other(T)
}

#[derive(Debug, Deserialize)]
pub struct SchemaObject {
    #[serde(rename = "type")]
    pub typ: Option<String>, // Can actually also be array of strings
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub properties: HashMap<String, RefOr<SchemaObject>>,
    pub items: Option<Box<RefOr<SchemaObject>>>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub enumerated: Option<Vec<Value>> // The items can be any type
}

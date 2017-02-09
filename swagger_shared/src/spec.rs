use serde_json::value::{Value, Map};

#[derive(Deserialize)]
pub struct Spec {
    pub swagger: String,
    pub info: Info,
    pub host: Option<String>,
    #[serde(rename = "basePath")]
    pub base_path: Option<String>,
    pub paths: Map<String, Value>,
    pub definitions: Option<Map<String, Value>>,
    pub parameters: Option<Map<String, Value>>,
}

#[derive(Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct Operation {}

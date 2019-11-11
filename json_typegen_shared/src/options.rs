use crate::hints::Hint;

/// Options for the code generation
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    pub output_mode: OutputMode,
    pub runnable: bool,
    pub use_default_for_missing_fields: bool,
    pub deny_unknown_fields: bool,
    pub(crate) allow_option_vec: bool,
    pub type_visibility: String,
    pub field_visibility: Option<String>,
    pub derives: String,
    pub(crate) hints: Vec<(String, Hint)>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            output_mode: OutputMode::Rust,
            runnable: false,
            use_default_for_missing_fields: false,
            deny_unknown_fields: false,
            allow_option_vec: false,
            type_visibility: "pub".into(),
            field_visibility: Some("pub".into()),
            derives: "Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize".into(),
            hints: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OutputMode {
    Rust,
    Typescript,
    Kotlin,
    JsonSchema,
    Shape,
}

impl OutputMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "rust" => Some(OutputMode::Rust),
            "typescript" => Some(OutputMode::Typescript),
            "kotlin" => Some(OutputMode::Kotlin),
            "json_schema" => Some(OutputMode::JsonSchema),
            "shape" => Some(OutputMode::Shape),
            _ => None,
        }
    }
}

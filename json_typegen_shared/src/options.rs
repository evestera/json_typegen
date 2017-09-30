use hints::{Hint};

/// Options for the code generation
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
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
            runnable: false,
            use_default_for_missing_fields: false,
            deny_unknown_fields: false,
            allow_option_vec: false,
            type_visibility: "".into(),
            field_visibility: None,
            derives: "Default, Debug, Clone, PartialEq, Serialize, Deserialize".into(),
            hints: Vec::new(),
        }
    }
}

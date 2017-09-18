use hints::{Hint};

#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    pub extern_crate: bool,
    pub runnable: bool,
    pub use_default_for_missing_fields: bool,
    pub deny_unknown_fields: bool,
    pub allow_option_vec: bool,
    pub type_visibility: String,
    pub field_visibility: Option<String>,
    pub derives: String,
    pub hints: Vec<(String, Hint)>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            extern_crate: false,
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

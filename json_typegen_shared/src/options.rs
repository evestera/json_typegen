use quote::{ Tokens, ToTokens };

use hints::{Hint};

#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    pub extern_crate: bool,
    pub runnable: bool,
    pub missing_fields: MissingFields,
    pub deny_unknown_fields: bool,
    pub allow_option_vec: bool,
    pub type_visibility: Visibility,
    pub field_visibility: FieldVisibility,
    pub derives: String,
    pub hints: Vec<(String, Hint)>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            extern_crate: false,
            runnable: false,
            missing_fields: MissingFields::Fail,
            deny_unknown_fields: false,
            allow_option_vec: false,
            type_visibility: Visibility::Private,
            field_visibility: FieldVisibility::Inherited,
            derives: "Default, Debug, Clone, PartialEq, Serialize, Deserialize".into(),
            hints: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MissingFields {
    Fail,
    UseDefault,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Visibility {
    Private,
    Pub,
    PubRestricted(String)
}

impl ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut Tokens) {
        use Visibility::*;
        match *self {
            Private => {},
            Pub => {
                tokens.append("pub");
            }
            PubRestricted(ref path) => {
                tokens.append("pub(");
                tokens.append(path);
                tokens.append(")");
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldVisibility {
    Inherited,
    Specified(Visibility)
}

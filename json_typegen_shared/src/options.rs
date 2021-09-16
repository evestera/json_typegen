use crate::hints::Hint;

/// Options for the code generation
///
/// Construct with `Options::default()`, and change any settings you care about.
#[non_exhaustive]
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
    pub property_name_format: Option<StringTransform>,
    pub(crate) hints: Vec<(String, Hint)>,
    pub unwrap: String,
    pub import_style: ImportStyle,
    pub collect_additional: bool,
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
            property_name_format: None,
            hints: Vec::new(),
            unwrap: "".into(),
            import_style: ImportStyle::AddImports,
            collect_additional: false,
        }
    }
}

#[cfg(feature = "option-parsing")]
impl Options {
    pub(crate) fn macro_default() -> Options {
        let mut options = Options::default();
        options.import_style = ImportStyle::QualifiedPaths;
        options
    }
}

/// How imports/external types should be handled by code generation
#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum ImportStyle {
    /// Add import/use statements for any external types used
    AddImports,
    /// Assume import/use statements already exist where the generated code will be inserted
    AssumeExisting,
    /// Use fully qualified paths for any external type used
    QualifiedPaths,
}

impl ImportStyle {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "add_imports" => Some(ImportStyle::AddImports),
            "assume_existing" => Some(ImportStyle::AssumeExisting),
            "qualified_paths" => Some(ImportStyle::QualifiedPaths),
            _ => None,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum OutputMode {
    Rust,
    Typescript,
    TypescriptTypeAlias,
    KotlinJackson,
    KotlinKotlinx,
    JsonSchema,
    Shape,
}

impl OutputMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "rust" => Some(OutputMode::Rust),
            "typescript" => Some(OutputMode::Typescript),
            "typescript/typealias" => Some(OutputMode::TypescriptTypeAlias),
            "kotlin" => Some(OutputMode::KotlinJackson),
            "kotlin/jackson" => Some(OutputMode::KotlinJackson),
            "kotlin/kotlinx" => Some(OutputMode::KotlinKotlinx),
            "json_schema" => Some(OutputMode::JsonSchema),
            "shape" => Some(OutputMode::Shape),
            _ => None,
        }
    }
}

// https://serde.rs/container-attrs.html rename_all:
// "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case",
// "SCREAMING_SNAKE_CASE", "kebab-case", "SCREAMING-KEBAB-CASE"

// Jackson JsonNaming PropertyNamingStrategy:
// KebabCaseStrategy, LowerCaseStrategy, SnakeCaseStrategy, UpperCamelCaseStrategy
#[derive(Debug, PartialEq, Clone)]
pub enum StringTransform {
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl StringTransform {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "lowercase" => Some(StringTransform::LowerCase),
            "uppercase" | "UPPERCASE" => Some(StringTransform::UpperCase),
            "pascalcase" | "uppercamelcase" | "PascalCase" => Some(StringTransform::PascalCase),
            "camelcase" | "camelCase" => Some(StringTransform::CamelCase),
            "snakecase" | "snake_case" => Some(StringTransform::SnakeCase),
            "screamingsnakecase" | "SCREAMING_SNAKE_CASE" => {
                Some(StringTransform::ScreamingSnakeCase)
            }
            "kebabcase" | "kebab-case" => Some(StringTransform::KebabCase),
            "screamingkebabcase" | "SCREAMING-KEBAB-CASE" => {
                Some(StringTransform::ScreamingKebabCase)
            }
            _ => None,
        }
    }
}

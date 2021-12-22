use crate::generation::json_schema::json_schema;
use crate::generation::typescript_type_alias::typescript_type_alias;
use crate::{Options, Shape};

pub type Code = String;

pub fn typescript_with_ajv(name: &str, shape: &Shape, options: Options) -> Code {
    let type_alias = typescript_type_alias(name, shape, options.clone());
    let schema = json_schema(name, shape, options);
    let schemaname = format!("{}Schema", name);

    format!(
        r#"import Ajv, {{JSONSchemaType}} from "ajv"
const ajv = new Ajv();

{type_alias}

export const {schemaname}: JSONSchemaType<{name}> = {schema};

export const validate{name} = ajv.compile({schemaname});
"#,
        type_alias = type_alias,
        schemaname = schemaname,
        schema = schema,
        name = name,
    )
}

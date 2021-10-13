use linked_hash_map::LinkedHashMap;

pub enum Value {
    Null,
    Bool(bool),
    // Number(f64),
    Str(&'static str),
    String(String),
    Array(Vec<Value>),
    Object(LinkedHashMap<String, Value>),
}

pub fn pretty_print_value(indent: usize, value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => format!("{}", b),
        Value::Str(s) => format!("\"{}\"", s),
        Value::String(s) => format!("\"{}\"", s),
        Value::Array(values) => {
            let mut code = "[\n".to_string();
            let len = values.len();
            for (i, val) in values.iter().enumerate() {
                code += &"  ".repeat(indent + 1);
                code += &pretty_print_value(indent + 1, val);
                if i != len - 1 {
                    code += ",";
                }
                code += "\n";
            }
            code += &"  ".repeat(indent);
            code += "]";
            code
        }
        Value::Object(map) => {
            let mut code = "{\n".to_string();
            let len = map.len();
            for (i, (key, val)) in map.iter().enumerate() {
                code += &"  ".repeat(indent + 1);
                code += &format!("\"{}\": ", key);
                code += &pretty_print_value(indent + 1, val);
                if i != len - 1 {
                    code += ",";
                }
                code += "\n";
            }
            code += &"  ".repeat(indent);
            code += "}";
            code
        }
    }
}

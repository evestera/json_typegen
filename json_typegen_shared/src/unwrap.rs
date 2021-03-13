use serde_json::{Value, json};

/// "Unwrap" JSON nodes so that the node(s) specified by the pointer is the new root(s)
pub fn unwrap(pointer: &str, value: Value) -> Vec<Value> {
    dbg!(pointer);
    let pointer_tokens: Vec<&str> = if pointer.is_empty() || pointer == "/" {
        return vec![value];
    } else {
        if pointer.starts_with('/') {
            pointer.split('/').skip(1).collect()
        } else {
            pointer.split('/').collect()
        }
    };

    if pointer_tokens.is_empty() {
        return vec![value];
    }

    let mut next: Vec<Value> = vec![value];
    for pointer_token in dbg!(pointer_tokens) {
        let mut work = Vec::new();
        std::mem::swap(&mut work, &mut next);
        for val in work {
            match val {
                Value::Object(mut values) => {
                    if let Some(y) = values.remove(pointer_token) {
                        next.push(y);
                    } else if pointer_token == "-" {
                        next.append(&mut values.into_iter().map(|(_k, v)| v).collect())
                    }
                }
                Value::Array(mut values) => {
                    if pointer_token == "-" {
                        next.append(&mut values)
                    } else if let Ok(i) = pointer_token.parse::<usize>() {
                        if i < values.len() {
                            next.push(values.remove(i));
                        }
                    }
                }
                Value::Null => {}
                Value::Bool(_) => {}
                Value::Number(_) => {}
                Value::String(_) => {}
            }
        }
    }

    return next;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap() {
        assert_eq!(unwrap("", json!(true)), vec![json!(true)]);
        assert_eq!(unwrap("/", json!(true)), vec![json!(true)]);
        assert_eq!(unwrap("/foo", json!({ "foo": true })), vec![json!(true)]);
        assert_eq!(unwrap("/foo/-/bar", json!({
            "foo": [
                { "bar": 1 },
                { "bar": 2 },
                { "nope": "hello" },
                { "bar": 3 },
            ]
        })), vec![json!(1), json!(2), json!(3)]);
    }
}

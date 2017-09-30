extern crate json_typegen_shared;
extern crate syn;

use json_typegen_shared::{codegen, Options};

/// Function to test AST equality, not string equality
fn code_output_test(name: &str, input: &str, expected: &str) {
    let res = codegen(name, input, Options::default());
    let output = res.unwrap();
    assert_eq!(
        syn::parse_items(&output),
        syn::parse_items(expected),
        "\n\nUnexpected output code:\n  input: {}\n  output:\n{}\n  expected: {}",
        input, output, expected);
}

#[test]
fn empty_object() {
    code_output_test(
        "Root",
        r##"
            {}
        "##,
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Root {}
        "##
    );
}

#[test]
fn point() {
    code_output_test(
        "Point",
        r##"
            {
                "x": 2,
                "y": 3
            }
        "##,
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Point {
                x: i64,
                y: i64,
            }
        "##
    );
}

#[test]
fn pub_point() {
    code_output_test(
        "pub Point",
        r##"
            {
                "x": 2,
                "y": 3
            }
        "##,
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct Point {
                pub x: i64,
                pub y: i64,
            }
        "##
    );
}

#[test]
fn optionals() {
    code_output_test(
        "Optional",
        r##"
            [
                {
                    "in_both": 5,
                    "missing": 5,
                    "has_null": 5
                },
                {
                    "in_both": 5,
                    "has_null": null,
                    "added": 5
                }
            ]
        "##,
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Optional {
                in_both: i64,
                missing: Option<i64>,
                has_null: Option<i64>,
                added: Option<i64>,
            }
        "##
    );
}

#[test]
fn fallback() {
    code_output_test(
        "Fallback",
        r##"
            [
                {
                    "only_null": null,
                    "conflicting": 5,
                    "empty_array": []
                },
                {
                    "only_null": null,
                    "conflicting": "five",
                    "empty_array": []
                }
            ]
        "##,
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Fallback {
                only_null: ::serde_json::Value,
                conflicting: ::serde_json::Value,
                empty_array: Vec<::serde_json::Value>,
            }
        "##
    );
}

#[test]
fn nesting() {
    code_output_test(
        "Root",
        r##"
            [
                {
                    "nested": {
                        "a": 5,
                        "doubly_nested": { "c": 10 }
                    },
                    "in_array": [{ "b": 5 }]
                }
            ]
        "##,
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Root {
                nested: Nested,
                in_array: Vec<InArray>,
            }

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Nested {
                a: i64,
                doubly_nested: DoublyNested,
            }

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct DoublyNested {
                c: i64,
            }

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct InArray {
                b: i64,
            }
        "##
    );
}

#[test]
fn tuple() {
    code_output_test(
        "Pagination",
        r##"
            [
                {
                    "pages": 1,
                    "items": 3
                },
                [
                    {
                        "name": "John"
                    },
                    {
                        "name": "James"
                    },
                    {
                        "name": "Jake"
                    }
                ]
            ]
        "##,
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Pagination {
                pages: i64 ,
                items: i64
            }

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Pagination2 {
                name : String
            }
        "##
    );
}

#[test]
fn rename() {
    code_output_test(
        "Renamed",
        r##"
            {
                "type": 5
            }
        "##,
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            struct Renamed {
                #[serde(rename = "type")]
                type_field: i64
            }
        "##
    );
}

use testsyn::{parse_str, Item};

use json_typegen_shared::{codegen, ImportStyle, Options};

/// Function to test AST equality, not string equality
fn code_output_test(name: &str, input: &str, expected: &str) {
    let mut options = Options::default();
    options.import_style = ImportStyle::AssumeExisting;
    let res = codegen(name, input, options);
    let output = res.unwrap();
    assert_eq!(
        // Wrapping in mod Foo { } since there is no impl Parse for Vec<Item>
        parse_str::<Item>(&format!("mod Foo {{ {} }}", &output)).unwrap(),
        parse_str::<Item>(&format!("mod Foo {{ {} }}", expected)).unwrap(),
        "\n\nUnexpected output code:\n  input: {}\n  output:\n{}\n  expected: {}",
        input,
        output,
        expected
    );
}

#[test]
fn empty_object() {
    code_output_test(
        "Root",
        // language=JSON
        r##"
            {}
        "##,
        // language=Rust
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct Root {}
        "##,
    );
}

#[test]
fn list_of_numbers() {
    code_output_test(
        "Numbers",
        // language=JSON
        r##"
            [1, 2, 3]
        "##,
        // language=Rust
        r##"
            pub type Numbers = Vec<i64>;
        "##,
    );
}

#[test]
fn point() {
    code_output_test(
        "Point",
        // language=JSON
        r##"
            {
                "x": 2,
                "y": 3
            }
        "##,
        // language=Rust
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct Point {
                pub x: i64,
                pub y: i64,
            }
        "##,
    );
}

#[test]
fn pub_crate_point() {
    code_output_test(
        "pub(crate) Point",
        // language=JSON
        r##"
            {
                "x": 2,
                "y": 3
            }
        "##,
        // language=Rust
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub(crate) struct Point {
                pub x: i64,
                pub y: i64,
            }
        "##,
    );
}

#[test]
fn optionals() {
    code_output_test(
        "Optionals",
        // language=JSON
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
        // language=Rust
        r##"
            pub type Optionals = Vec<Optional>;

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct Optional {
                pub in_both: i64,
                pub missing: Option<i64>,
                pub has_null: Option<i64>,
                pub added: Option<i64>,
            }
        "##,
    );
}

#[test]
fn fallback() {
    code_output_test(
        "FallbackExamples",
        // language=JSON
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
        // language=Rust
        r##"
            pub type FallbackExamples = Vec<FallbackExample>;

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct FallbackExample {
                pub only_null: Value,
                pub conflicting: Value,
                pub empty_array: Vec<Value>,
            }
        "##,
    );
}

#[test]
fn nesting() {
    code_output_test(
        "NestedTypes",
        // language=JSON
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
        // language=Rust
        r##"
            pub type NestedTypes = Vec<NestedType>;

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct NestedType {
                pub nested: Nested,
                pub in_array: Vec<InArray>,
            }

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct Nested {
                pub a: i64,
                pub doubly_nested: DoublyNested,
            }

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct DoublyNested {
                pub c: i64,
            }

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct InArray {
                pub b: i64,
            }
        "##,
    );
}

#[test]
fn tuple() {
    code_output_test(
        "Pagination",
        // language=JSON
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
        // language=Rust
        r##"
            pub type Pagination = (Pagination2, Vec<Pagination3>);

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct Pagination2 {
                pub pages: i64,
                pub items: i64,
            }

            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct Pagination3 {
                pub name : String,
            }
        "##,
    );
}

#[test]
fn rename() {
    code_output_test(
        "Renamed",
        // language=JSON
        r##"
            {
                "type": 5
            }
        "##,
        // language=Rust
        r##"
            #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct Renamed {
                #[serde(rename = "type")]
                pub type_field: i64,
            }
        "##,
    );
}

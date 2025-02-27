use json_typegen_shared::{codegen, ImportStyle, Options, OutputMode};

/// Function to test AST equality, not string equality
fn code_output_test(name: &str, input: &str, expected: &str) {
    let mut options = Options::default();
    options.import_style = ImportStyle::AssumeExisting;
    options.output_mode = OutputMode::PythonPydantic;
    let res = codegen(name, input, options);
    let output = res.unwrap();
    let expected = &expected[1..];
    assert_eq!(
        output, expected,
        "\n\nUnexpected output code:\n  input: {}\n  output:\n{}\n  expected: {}",
        input, output, expected
    );
}

#[test]
fn empty_object() {
    code_output_test(
        "Root",
        r##"
            {}
        "##,
        r##"
class Root(BaseModel):
    pass
"##,
    );
}

#[test]
fn list_of_numbers() {
    code_output_test(
        "Numbers",
        r##"
            [1, 2, 3]
        "##,
        "
Numbers = list[int]
",
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
class Point(BaseModel):
    x: int
    y: int
"##,
    );
}

#[test]
fn optionals() {
    code_output_test(
        "Opts",
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
class Opt(BaseModel):
    in_both: int
    missing: Optional[int]
    has_null: Optional[int]
    added: Optional[int]


Opts = list[Opt]
"##,
    );
}

#[test]
fn fallback() {
    code_output_test(
        "FallbackExamples",
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
class FallbackExample(BaseModel):
    only_null: Any
    conflicting: Any
    empty_array: list[Any]


FallbackExamples = list[FallbackExample]
"##,
    );
}

#[test]
fn nesting() {
    code_output_test(
        "NestedTypes",
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
class DoublyNested(BaseModel):
    c: int


class Nested(BaseModel):
    a: int
    doubly_nested: DoublyNested


class InArray(BaseModel):
    b: int


class NestedType(BaseModel):
    nested: Nested
    in_array: list[InArray]


NestedTypes = list[NestedType]
"##,
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
class Pagination2(BaseModel):
    pages: int
    items: int


class Pagination3(BaseModel):
    name: str


Pagination = tuple[Pagination2, list[Pagination3]]
"##,
    );
}

#[test]
fn rename() {
    code_output_test(
        "Renamed",
        r##"
            {
                "class": 5
            }
        "##,
        r##"
class Renamed(BaseModel):
    class_field: int = Field(alias="class")
"##,
    );
}

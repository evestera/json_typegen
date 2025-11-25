use crate::hints::Hints;
use crate::shape::common_shape;
use crate::{JTError, Options, Shape};
use linked_hash_map::LinkedHashMap;
use std::io::{BufReader, Read};
use xml::EventReader;
use xml::reader::XmlEvent;

/// Recursively apply a transformation function to a shape and all its children.
fn recursively_apply(shape: Shape, f: &dyn Fn(Shape) -> Shape) -> Shape {
    // If the shape is a nested shape, apply the transformation to the nested shape.
    let after_recursion = match shape {
        Shape::Struct { fields } => Shape::Struct {
            fields: fields
                .into_iter()
                .map(|(k, v)| (k, recursively_apply(v, f)))
                .collect(),
        },
        Shape::VecT { elem_type } => Shape::VecT {
            elem_type: Box::new(recursively_apply(*elem_type, f)),
        },
        Shape::MapT { val_type } => Shape::MapT {
            val_type: Box::new(recursively_apply(*val_type, f)),
        },
        Shape::Tuple(shapes, n) => Shape::Tuple(
            shapes
                .into_iter()
                .map(|shape| recursively_apply(shape, f))
                .collect(),
            n,
        ),
        Shape::Optional(t) => Shape::Optional(Box::new(recursively_apply(*t, f))),
        Shape::Nullable(t) => Shape::Nullable(Box::new(recursively_apply(*t, f))),
        _ => shape,
    };

    f(after_recursion)
}

fn apply_transformations(shape: Shape, transformations: &[&dyn Fn(Shape) -> Shape]) -> Shape {
    transformations
        .iter()
        .fold(shape, |shape, f| recursively_apply(shape, f))
}

pub fn xml_to_shape<R: Read>(read: R, options: &Options, hints: &Hints) -> Result<Shape, JTError> {
    let mut parser = EventReader::new(BufReader::with_capacity(128 * 1024, read));
    let inferred = partial_xml_to_shape(&mut parser, None, LinkedHashMap::new(), options, hints)
        .map_err(JTError::XmlParsingError)?;

    let transformed = apply_transformations(
        inferred,
        &[
            &|shape| {
                if let Shape::Struct { fields } = &shape {
                    if fields.len() == 1 && fields.contains_key("@content") {
                        return fields["@content"].clone();
                    }
                }
                shape
            },
            &|shape| {
                if let Shape::Struct { fields } = &shape {
                    if fields.is_empty() {
                        return Shape::Bottom;
                    }
                }
                shape
            },
        ],
    );

    Ok(transformed)
}

fn partial_xml_to_shape<R: Read>(
    parser: &mut EventReader<R>,
    expected_end_element: Option<&str>,
    mut attributes: LinkedHashMap<String, Shape>,
    options: &Options,
    hints: &Hints,
) -> Result<Shape, String> {
    let mut children = LinkedHashMap::<String, Shape>::new();
    let mut has_content = false;
    loop {
        let e = parser.next();
        match e {
            Ok(e) => match e {
                XmlEvent::StartDocument { .. } => {}
                XmlEvent::EndDocument => {
                    return if let Some(expected_end_element) = expected_end_element {
                        Err(format!(
                            "Expected end element {:?} but found end of document",
                            expected_end_element
                        ))
                    } else {
                        Ok(Shape::Struct { fields: children })
                    };
                }
                XmlEvent::ProcessingInstruction { .. } => {}
                XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace: _,
                } => {
                    let mut attribute_shapes = LinkedHashMap::<String, Shape>::new();
                    for attribute in attributes {
                        attribute_shapes
                            .insert("+".to_string() + &attribute.name.local_name, Shape::StringT);
                    }
                    let shape = partial_xml_to_shape(
                        parser,
                        Some(&name.local_name),
                        attribute_shapes,
                        options,
                        hints,
                    )?;
                    match children.entry(name.local_name) {
                        linked_hash_map::Entry::Occupied(mut entry) => {
                            let slot = entry.get_mut();
                            *slot = common_shape(std::mem::replace(slot, Shape::Bottom), shape);
                        }
                        linked_hash_map::Entry::Vacant(entry) => {
                            entry.insert(shape);
                        }
                    }
                }
                XmlEvent::EndElement { name } => {
                    return match expected_end_element {
                        Some(expected_end_element) => {
                            if name.local_name == expected_end_element {
                                if has_content {
                                    children.insert("@content".to_string(), Shape::StringT);
                                }
                                children.into_iter().for_each(|(k, v)| {
                                    attributes.insert(k, v);
                                });
                                Ok(Shape::Struct { fields: attributes })
                            } else {
                                Err(format!(
                                    "Expected end element {:?} but found end element {:?}",
                                    expected_end_element, name.local_name
                                ))
                            }
                        }
                        None => Err(format!(
                            "Expected end of document but found end element {:?}",
                            name.local_name
                        )),
                    };
                }
                XmlEvent::CData(_) => {
                    has_content = true;
                }
                XmlEvent::Comment(_) => {}
                XmlEvent::Characters(_) => {
                    has_content = true;
                }
                XmlEvent::Whitespace(_) => {}
            },
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, OutputMode, codegen_from_shape};

    #[test]
    fn test_sql_to_shape() {
        let shape = xml_to_shape(
            r#"
                <users href="//example.com/users">
                    <user>
                        <id>1</id>
                        <name>John Doe</name>
                        <age>30</age>
                    </user>
                    <user>
                        <id>2</id>
                        <name>Jane Doe</name>
                    </user>
                </users>
            "#
            .as_bytes(),
            &Options::default(),
            &Hints::new(),
        )
        .unwrap();
        let output = codegen_from_shape(
            "test",
            &shape,
            Options {
                output_mode: OutputMode::TypescriptTypeAlias,
                ..Options::default()
            },
        )
        .unwrap();
        println!("{}", output);
    }
}

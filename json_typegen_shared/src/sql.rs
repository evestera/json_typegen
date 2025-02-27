use crate::Shape;
use sqlparser::ast::{ColumnDef, ColumnOption, DataType, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

pub fn sql_to_shape(input: &str) -> Result<Vec<(String, Shape)>, String> {
    let dialect = GenericDialect {};
    let ast: Vec<Statement> = Parser::parse_sql(&dialect, input).map_err(|e| e.to_string())?;
    Ok(ast
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::CreateTable { name, columns, .. } => Some((
                name.to_string(),
                Shape::Struct {
                    fields: columns
                        .iter()
                        .map(|column: &ColumnDef| {
                            (column.name.to_string(), shape_for_column(column))
                        })
                        .collect(),
                },
            )),
            _ => None,
        })
        .collect())
}

fn shape_for_column(column: &ColumnDef) -> Shape {
    let base_shape = match column.data_type {
        DataType::Character(_) |
        DataType::Char(_) |
        DataType::CharacterVarying(_) |
        DataType::CharVarying(_) |
        DataType::Varchar(_) |
        DataType::Nvarchar(_) |
        DataType::Text |
        DataType::String => Shape::StringT,
        // DataType::Uuid => {}
        // DataType::CharacterLargeObject(_) => {}
        // DataType::CharLargeObject(_) => {}
        // DataType::Clob(_) => {}
        // DataType::Binary(_) => {}
        // DataType::Varbinary(_) => {}
        // DataType::Blob(_) => {}
        // DataType::Numeric(_) => {}
        // DataType::Decimal(_) => {}
        // DataType::BigNumeric(_) => {}
        // DataType::BigDecimal(_) => {}
        // DataType::Dec(_) => {}
        // DataType::Float(_) => {}
        DataType::TinyInt(_) |
        DataType::UnsignedTinyInt(_) |
        DataType::SmallInt(_) |
        DataType::UnsignedSmallInt(_) |
        DataType::MediumInt(_) |
        DataType::UnsignedMediumInt(_) |
        DataType::Int(_) |
        DataType::Integer(_) |
        DataType::UnsignedInt(_) |
        DataType::UnsignedInteger(_) |
        DataType::BigInt(_) |
        DataType::UnsignedBigInt(_) => Shape::Integer,
        // DataType::Real => {}
        // DataType::Double => {}
        // DataType::DoublePrecision => {}
        DataType::Boolean => Shape::Bool,
        DataType::Date |
        // DataType::Time(_, _) => {}
        DataType::Datetime(_) |
        DataType::Timestamp(_, _) => Shape::Opaque("Date".to_string()),
        // DataType::Interval => {}
        // DataType::JSON => {}
        // DataType::Regclass => {}
        // DataType::Bytea => {}
        // DataType::Custom(ObjectName(idents), _) => {}
        // DataType::Array(data_type_opt) => {
        //
        // }
        // DataType::Enum(_) => {}
        // DataType::Set(_) => {}
        _ => { Shape::Any }
    };
    let nullable = !column
        .options
        .iter()
        .any(|option| matches!(option.option, ColumnOption::NotNull));
    if nullable {
        base_shape.into_nullable()
    } else {
        base_shape
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{codegen_from_shape, Options, OutputMode};

    #[test]
    fn test_sql_to_shape() {
        let output = sql_to_shape(
            r#"
                CREATE TABLE users (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    age INT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
            "#,
        )
        .unwrap()
        .iter()
        .map(|(name, shape)| {
            codegen_from_shape(
                name,
                shape,
                Options {
                    output_mode: OutputMode::ZodSchema,
                    ..Options::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n");
        println!("{}", output);
    }
}

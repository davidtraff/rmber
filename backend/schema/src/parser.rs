use std::{collections::{HashMap, HashSet}, convert::{TryFrom, TryInto}};

use super::{Namespace, Point, PointType};
use pest::{
    error::{Error, ErrorVariant},
    iterators::Pair,
    Parser,
};

#[derive(Parser)]
#[grammar = "schema.pest"]
struct SchemaParser;

pub fn parse(input: &str) -> Result<Vec<Namespace>, Error<Rule>> {
    let result = SchemaParser::parse(Rule::root, input)?;

    let mut namespaces = HashMap::new();
    for namespace in result.into_iter() {
        traverse_tree(&mut namespaces, namespace)?;
    }

    let namespaces = namespaces
        .into_iter()
        .map(|(name, points)| Namespace { name, points })
        .collect();

    Ok(namespaces)
}

fn traverse_tree(namespaces: &mut HashMap<String, Vec<Point>>, pair: Pair<Rule>) -> Result<(), Error<Rule>> {
    let contents = pair.into_inner();
    let mut name = None;
    let mut points: HashSet<Point> = HashSet::new();

    for inner in contents {
        match inner.as_rule() {
            Rule::namespace => {
                traverse_tree(namespaces, inner)?;
            }
            Rule::identifier => {
                name = Some(String::from(inner.as_str()));
            }
            Rule::point => {
                let mut point = convert_point(inner)?;
                
                if let Some(previous) = points.take(&point) {
                    previous.types.into_iter().for_each(|pt| {
                        point.types.insert(pt);
                    });
                }

                assert!(points.insert(point));
            }
            _ => unimplemented!(),
        }
    }

    if let Some(name) = name {
        if let Some(values) = namespaces.get_mut(&name) {
            points.into_iter().for_each(|p| values.push(p));
        } else {
            namespaces.insert(name, points.into_iter().collect());
        }
    }

    Ok(())
}

fn convert_point(point: Pair<Rule>) -> Result<Point, Error<Rule>> {
    assert_eq!(Rule::point, point.as_rule());
    let mut name = None;
    let mut types = HashSet::new();

    for inner in point.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                name = Some(String::from(inner.as_str()));
            }
            Rule::point_type => {
                let value = inner.into_inner().next().expect("Should never happen.");

                types.insert(value.try_into()?);
            }
            _ => unimplemented!(),
        }
    }

    Ok(Point {
        name: name.unwrap(),
        types,
    })
}

impl TryFrom<Pair<'_, Rule>> for super::PointType {
    type Error = Error<Rule>;

    fn try_from(value: Pair<Rule>) -> Result<Self, Self::Error> {
        match value.as_str() {
            "boolean" | "Boolean" => Ok(PointType::Boolean),
            "blob" | "Blob" => Ok(PointType::Blob),
            "string" | "String" => Ok(PointType::String),
            "u8" | "U8" => Ok(PointType::U8),
            "i8" | "I8" => Ok(PointType::I8),
            "u16" | "U16" => Ok(PointType::U16),
            "i16" | "I16" => Ok(PointType::I16),
            "u32" | "U32" => Ok(PointType::U32),
            "i32" | "I32" => Ok(PointType::I32),
            "u64" | "U64" => Ok(PointType::U64),
            "i64" | "I64" => Ok(PointType::I64),
            "f32" | "F32" => Ok(PointType::F32),
            "f64" | "F64" => Ok(PointType::F64),
            _ => Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: String::from("Invalid type-name"),
                },
                value.as_span(),
            )),
        }
    }
}

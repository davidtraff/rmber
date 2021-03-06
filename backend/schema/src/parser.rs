use std::{
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
};

use super::{Namespace, Point, PointType};
use pest::{
    error::ErrorVariant,
    iterators::Pair,
    Parser,
};

pub use pest::error::Error;

pub const NS_DIVIDER: &str = "/";

#[derive(Parser)]
#[grammar = "schema.pest"]
struct SchemaParser;

pub fn parse(input: &str) -> Result<Vec<Namespace>, Error<Rule>> {
    let result = SchemaParser::parse(Rule::root, input)?;

    let mut namespaces = HashMap::new();
    for namespace in result.into_iter() {
        traverse_tree(&mut namespaces, None, namespace)?;
    }

    let namespaces = namespaces
        .into_iter()
        .map(|(name, points)| Namespace { name, points })
        .collect();

    Ok(namespaces)
}

fn traverse_tree(
    namespaces: &mut HashMap<String, HashSet<Point>>,
    parent: Option<String>,
    pair: Pair<Rule>,
) -> Result<(), Error<Rule>> {
    let contents = pair.into_inner();
    let mut name = None;
    let mut point_rules: Vec<Pair<Rule>> = vec![];

    for inner in contents {
        match inner.as_rule() {
            Rule::namespace => {
                traverse_tree(namespaces, name.clone(), inner)?;
            }
            Rule::identifier => {
                if let Some(parent) = &parent {
                    name = Some(format!("{}{}{}", parent, NS_DIVIDER, inner.as_str()));
                } else {
                    name = Some(String::from(inner.as_str()));
                }
            }
            Rule::point => {
                point_rules.push(inner);
            }
            _ => unimplemented!(),
        }
    }

    if let Some(name) = name {
        let mut points: HashSet<Point> = HashSet::new();

        for point in point_rules {
            let mut point = convert_point(&name, point)?;

            if let Some(previous) = points.take(&point) {
                previous.types.into_iter().for_each(|pt| {
                    point.types.insert(pt);
                });
            }

            assert!(points.insert(point));
        }

        if let Some(values) = namespaces.get_mut(&name) {
            // If we already have an equally named point in this namespace we merge it.
            points.into_iter().for_each(|mut p| {
                if let Some(previous) = values.take(&p) {
                    p.merge(previous);
                }

                values.insert(p);
            });
        } else {
            namespaces.insert(name, points);
        }
    }

    Ok(())
}

fn convert_point(namespace: &str, point: Pair<Rule>) -> Result<Point, Error<Rule>> {
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

    Ok(Point::new(name.unwrap(), String::from(namespace), types))
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

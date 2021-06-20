use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
};

use super::{Namespace, Point, PointType, schema::Schema};
use indextree::{Arena, NodeId};
use pest::{
    error::{Error, ErrorVariant},
    iterators::Pair,
    Parser,
};

pub const NS_DIVIDER: &str = ".";

#[derive(Parser)]
#[grammar = "schema.pest"]
struct SchemaParser;

pub fn parse(input: &str) -> Result<Schema, Error<Rule>> {
    let result = SchemaParser::parse(Rule::root, input)?;

    let mut arena = Arena::new();
    for namespace in result.into_iter() {
        traverse_tree(&mut arena, None, namespace)?;
    }

    let schema = Schema::new(arena);

    Ok(schema)
}

fn traverse_tree(
    arena: &mut Arena<Namespace>,
    parent: Option<NodeId>,
    pair: Pair<Rule>,
) -> Result<(), Error<Rule>> {
    let contents = pair.into_inner();

    let mut points: HashSet<Point> = HashSet::new();
    let mut name = None;
    let mut descendants = vec![];

    for inner in contents {
        match inner.as_rule() {
            Rule::namespace => {
                descendants.push(inner);
            }
            Rule::identifier => {
                if let Some(parent) = &parent {
                    let parent_node = arena.get(*parent).unwrap();
                    name = Some(format!("{}{}{}", parent_node.get().name, NS_DIVIDER, inner.as_str()));
                } else {
                    name = Some(String::from(inner.as_str()));
                }
            }
            Rule::point => {
                let mut point = convert_point(inner)?;

                if let Some(previous) = points.take(&point) {
                    point.merge(previous);
                }

                assert!(points.insert(point));
            }
            _ => unimplemented!(),
        };
    }

    if let Some(name) = name {
        let namespace = Namespace {
            name: name.clone(),
            points,
        };

        let ns;
        match parent {
            Some(parent) => {
                // If we have a parent and we find a child with the same name is this current
                // namespace, we want to merge it. Otherwise just place a new namespace under the parent.
                let existing = parent
                    .children(&arena)
                    .find(|id| arena.get(*id).unwrap().get().name.eq(&name));

                if let Some(node_id) = existing {
                    ns = node_id;

                    let node = arena.get_mut(node_id).unwrap();
                    node.get_mut().combine(namespace);
                } else {
                    ns = arena.new_node(namespace);
                    parent.append(ns, arena);
                }
            }
            None => {
                ns = arena.new_node(namespace);
            }
        }

        for child in descendants {
            traverse_tree(arena, Some(ns), child)?
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

use std::collections::HashSet;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod parser;
mod schema;
mod subscription;

pub use parser::*;
pub use subscription::*;
pub use schema::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PointType {
    Boolean,
    Blob,
    String,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
}

#[derive(Debug, Eq)]
pub struct Point {
    pub types: HashSet<PointType>,
    pub name: String,
}

impl Point {
    pub fn merge(&mut self, other: Point) {
        other.types.into_iter().for_each(|p| {
            self.types.insert(p);
        });
    }
}

impl std::hash::Hash for Point {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

#[derive(Debug)]
pub struct Namespace {
    pub name: String,
    pub points: HashSet<Point>,
}

impl Namespace {
    #[allow(dead_code)]
    fn merge(self, mut other: Namespace) {
        for mut point in self.points.into_iter() {
            if let Some(other_point) = other.points.take(&point) {
                point.merge(other_point);
            }
        }
    }

    #[allow(dead_code)]
    fn combine(&mut self, other: Namespace) {
        for point in other.points.into_iter() {
            if let Some(mut p) = self.points.take(&point) {
                p.merge(point);
                self.points.insert(p);
            }
        }
    }

    pub fn new(name: String) -> Self {
        Namespace {
            name,
            points: HashSet::new(),
        }
    }
}

impl PartialEq for Namespace {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

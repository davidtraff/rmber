use std::collections::HashSet;

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod parser;

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
    pub points: Vec<Point>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = String::from("
            first {
                - asd: u8 | string
                - asd: blob
                - asd1: u8 | string
                - asd2: u8 | string
            }

            second {
                - asd: u8 | string
            }
        ");

        let result = parser::parse(&data).unwrap();

        assert_eq!(2, result.len());
    }
}

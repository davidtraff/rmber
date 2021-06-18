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
    fn merge(self, mut other: Namespace) {
        for mut point in self.points.into_iter() {
            if let Some(other_point) = other.points.take(&point) {
                point.merge(other_point);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_test() {
        let data = String::from(
            "
            first {
                - asd: u8 | string
                - asd: blob
                - asd1: u8 | string
                - asd2: u8 | u16
            }

            second{nested{-asd:u8|string}}
        ",
        );

        let result = parser::parse(&data).unwrap();

        assert_eq!(3, result.len());

        let first = check_for_ns(&result, "first", 3);
        let asd = check_for_point(first, "asd", 3);
        check_for_point_type(asd, PointType::U8);
        check_for_point_type(asd, PointType::String);
        check_for_point_type(asd, PointType::Blob);

        let asd1 = check_for_point(first, "asd1", 2);
        check_for_point_type(asd1, PointType::U8);
        check_for_point_type(asd1, PointType::String);

        let asd2 = check_for_point(first, "asd2", 2);
        check_for_point_type(asd2, PointType::U8);
        check_for_point_type(asd2, PointType::U16);

        check_for_ns(&result, "second", 0);

        let nested = check_for_ns(&result, "second.nested", 1);
        let asd = check_for_point(nested, "asd", 2);
        check_for_point_type(asd, PointType::U8);
        check_for_point_type(asd, PointType::String);
    }

    #[test]
    fn merge_namespaces() {
        let data = String::from(
            "
            first {
                - asd: u8 | string
                - asd1: u8 | string
                - asd2: u8 | u16

                second {
                    - test: u32
                }
            }

            first {
                - asd: blob

                second {
                    - test: string
                }
            }
        ",
        );

        let result = parser::parse(&data).unwrap();

        dbg!(&result);

        assert_eq!(2, result.len());

        let first = check_for_ns(&result, "first", 3);
        let asd = check_for_point(first, "asd", 3);
        check_for_point_type(asd, PointType::U8);
        check_for_point_type(asd, PointType::String);
        check_for_point_type(asd, PointType::Blob);

        let asd1 = check_for_point(first, "asd1", 2);
        check_for_point_type(asd1, PointType::U8);
        check_for_point_type(asd1, PointType::String);

        let asd2 = check_for_point(first, "asd2", 2);
        check_for_point_type(asd2, PointType::U8);
        check_for_point_type(asd2, PointType::U16);

        let first_second = check_for_ns(&result, "first.second", 1);
        let test = check_for_point(first_second, "test", 2);
        check_for_point_type(test, PointType::U32);
        check_for_point_type(test, PointType::String);
    }

    fn check_for_ns<'a>(
        ns: &'a Vec<Namespace>,
        name: &str,
        expected_count: usize,
    ) -> &'a Namespace {
        let item = ns.iter().find(|x| x.name.eq(name)).expect(name);

        assert_eq!(item.points.len(), expected_count);

        item
    }

    fn check_for_point<'a>(ns: &'a Namespace, name: &str, expected_count: usize) -> &'a Point {
        let point = ns.points.iter().find(|x| x.name.eq(name)).expect(name);

        assert_eq!(point.types.len(), expected_count);

        point
    }

    fn check_for_point_type(point: &Point, pt: PointType) {
        point
            .types
            .iter()
            .find(|x| (*x).eq(&pt))
            .expect(&format!("{:?} {:?}", point.name, pt));
    }
}

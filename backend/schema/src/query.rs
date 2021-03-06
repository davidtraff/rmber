use std::collections::HashSet;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

#[derive(Debug)]
pub struct QuerySet {
    points: HashSet<String>,
    globset: GlobSet,
}

impl QuerySet {
    pub fn new<TSource>(source: TSource) -> Result<Self, globset::Error>
    where
        TSource: IntoIterator<Item = String>,
    {
        let points = source.into_iter().collect();
        let globset = build_glob_matcher(&points)?;

        Ok(QuerySet { points, globset })
    }

    pub fn single(source: &str) -> Result<Self, globset::Error> {
        let points = std::iter::once(String::from(source)).collect();
        let globset = build_glob_matcher(&points)?;

        Ok(QuerySet { points, globset })
    }

    pub fn from_string(source: String) -> Result<Self, globset::Error> {
        let points = std::iter::once(source).collect();
        let globset = build_glob_matcher(&points)?;

        Ok(QuerySet { points, globset })
    }

    pub fn empty() -> Self {
        QuerySet {
            points: HashSet::new(),
            globset: build_glob_matcher(&HashSet::new()).unwrap(),
        }
    }

    pub fn insert_point(&mut self, new_point: &str) -> Result<(), globset::Error> {
        let new_point = String::from(new_point);

        self.points.insert(new_point);

        self.globset = build_glob_matcher(&self.points)?;

        Ok(())
    }

    pub fn matches(&self, candidate: &str) -> bool {
        self.globset.is_match(candidate)
    }
}

fn build_glob_matcher(source: &HashSet<String>) -> Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();

    for item in source {
        let matcher = GlobBuilder::new(item)
            .case_insensitive(true)
            .literal_separator(true)
            .backslash_escape(true)
            .build()?;

        builder.add(matcher);
    }

    Ok(builder.build()?)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn it_works() {
        let source = vec![
            String::from("some_namespace/*"),
            String::from("some_other_namespace/**/specific_point")
        ];

        let set = QuerySet::new(source).unwrap();

        assert!(set.matches("some_namespace/a_point"));
        assert!(!set.matches("some_namespace/nested/a_point"));

        assert!(set.matches("some_other_namespace/nested/very/deep/specific_point"));
        assert!(set.matches("some_other_namespace/specific_point"));
        assert!(!set.matches("some_other_namespace/a_point"));
    }
}

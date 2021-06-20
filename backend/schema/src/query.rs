use crate::NS_DIVIDER;

pub const WILDCARD: &str = "*";

pub struct QueryPart<'a>(&'a str);

impl<'a> QueryPart<'a> {
    pub fn is_wildcard(&self) -> bool {
        self.0.eq(WILDCARD)
    }

    pub fn matches(&self, other: &QueryPart) -> bool {
        self.is_wildcard() || other.is_wildcard() || self.0.eq(other.0)
    }
}

pub struct Query<'a> {
    value: &'a str,
}

impl<'a> Query<'a> {
    pub fn new(value: &'a str) -> Query<'a> {
        Query {
            value,
        }
    }

    pub fn get_parts(&'a self) -> impl Iterator<Item = QueryPart<'a>> {
        self.value.split(NS_DIVIDER).map(|part| QueryPart(part))
    }

    pub fn matches_exact(&self, other: &str) -> bool {
        self.value.eq(other)
    }

    pub fn matches_partial(&self, other: &str) -> bool {
        let other = Query::new(other);
        let other_parts = other.get_parts();

        for (lhs, rhs) in self.get_parts().zip(other_parts) {
            if !lhs.matches(&rhs) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn matches_exact() {
        let query = Query::new("test.namespace");

        assert!(query.matches_exact("test.namespace"));

        assert!(!query.matches_exact("test"));
        assert!(!query.matches_exact("test."));
        assert!(!query.matches_exact("*"));
    }

    #[test]
    pub fn matches_partial() {
        let query = Query::new("test.namespace");

        assert!(query.matches_partial("test"));
        assert!(query.matches_partial("test.namespace"));
        assert!(query.matches_partial("test.namespace.inner"));

        assert!(!query.matches_partial("asd"));
        assert!(!query.matches_partial("asd.test"));
        assert!(!query.matches_partial("asd.namespace"));
        assert!(!query.matches_partial("asd.test.namespace"));
    }

    #[test]
    pub fn matches_partial_wildcard() {
        let query = Query::new("test.namespace");

        assert!(query.matches_partial("*"));
        assert!(query.matches_partial("*.namespace"));
        assert!(query.matches_partial("test.*"));
        assert!(query.matches_partial("test.*.asd"));

        assert!(!query.matches_partial("*.asd"));

        let query = Query::new("test.namespace.inner");

        assert!(query.matches_partial("test.namespace.inner"));
        assert!(query.matches_partial("test.*.inner"));

        assert!(!query.matches_partial("test.*.asd"));
    }
}

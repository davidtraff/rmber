use protocol::{Key, StringKey, Value};
use schema::{Error, Point, PointType, QuerySet, Rule, Schema, parse};
use async_trait::async_trait;

pub mod rocksdb;

#[async_trait(?Send)]
pub trait Store<TKey>
where
    TKey: Key,
{
    async fn store_value(&mut self, key: TKey, value: Value) -> Result<(), std::io::Error>;

    async fn get_value(&mut self, key: TKey) -> Option<Value>;
}

pub struct ValueStore<TStore>
where
    TStore: Store<StringKey>,
{
    store: TStore,
    schema: Schema,
}

impl<TStore> ValueStore<TStore>
where
    TStore: Store<StringKey>,
{
    pub fn new(store: TStore) -> Self {
        ValueStore {
            store,
            schema: Schema::empty(),
        }
    }

    pub fn build_schema<TIter>(&mut self, source: TIter) -> Result<(), Error<Rule>>
    where
        TIter: Iterator<Item = String>,
    {
        let mut schema = String::new();

        source.for_each(|part| schema.push_str(&part));

        let namespaces = parse(&schema)?;

        self.schema = Schema::new(namespaces);

        Ok(())
    }

    pub fn query<'a>(&'a self, query: &str) -> Result<Vec<&'a Point>, String> {
        let query = match QuerySet::from_string(query.into()) {
            Ok(q) => q,
            Err(e) => return Err(e.to_string()),
        };

        let result = self.schema
            .points()
            .filter(|p| query.matches(&p.full_name))
            .collect();

        Ok(result)
    }

    pub fn query_single<'a>(&'a self, query: &str) -> Option<&'a Point> {
        // TODO: If the iterative search becomes too slow consider using some tree/binary-search based container.
        self.schema.points().find(|p| p.full_name.eq(query))
    }

    pub async fn update_point(&mut self, key: StringKey, new_value: Value) -> Result<(), std::io::Error> {
        use std::io::{Error, ErrorKind};

        let point = match self.query_single(key.as_str()) {
            Some(p) => p,
            None => return Err(Error::new(ErrorKind::NotFound, "Invalid point.")),
        };

        let value_type = to_point_type(&new_value);

        if !point.types.contains(&value_type) {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid point-type."));
        }

        self.store.store_value(key, new_value).await
    }
}

fn to_point_type(value: &Value) -> PointType {
    match value {
        Value::Boolean(_) => PointType::Boolean,
        Value::Blob(_) => PointType::Blob,
        Value::String(_) => PointType::String,
        Value::U8(_) => PointType::U8,
        Value::I8(_) => PointType::I8,
        Value::U16(_) => PointType::U16,
        Value::I16(_) => PointType::I16,
        Value::U32(_) => PointType::U32,
        Value::I32(_) => PointType::I32,
        Value::U64(_) => PointType::U64,
        Value::I64(_) => PointType::I64,
        Value::F32(_) => PointType::F32,
        Value::F64(_) => PointType::F64,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

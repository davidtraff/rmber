use async_trait::async_trait;
use std::io::{Cursor, Error, ErrorKind};

use protocol::{Key, Value};
use crate::ValueStore;

use super::Store;

pub use rocksdb::DB;

#[async_trait(?Send)]
impl<TKey> Store<TKey> for DB
    where
        TKey: Key + 'static
{
    async fn store_value(&mut self, key: &TKey, value: &Value) -> Result<(), Error> {
        let mut data = vec![];
        let mut cursor = Cursor::new(&mut data);

        value.write_to(&mut cursor).await?;

        match self.put(key.as_slice(), data) {
            Ok(_) => Ok(()),
            Err(e) => Err(convert_err(e))
        }
    }

    async fn get_value(&mut self, key: &TKey) -> Option<Value> {
        match self.get_pinned(key.as_slice()) {
            Ok(data) => match data {
                Some(data) => {
                    let mut cursor = Cursor::new(data);
                    
                    Value::read_from(&mut cursor).await.ok()
                },
                None => None,
            },
            Err(_) => None,
        }
    }
}

pub fn create_rocksdb(path: &str) -> ValueStore<DB> {
    let db = DB::open_default(path).unwrap();

    ValueStore::new(db)
}

fn convert_err(err: rocksdb::Error) -> Error {
    Error::new(ErrorKind::Other, err.to_string())
}

use std::io::{Cursor, Error, ErrorKind};

use protocol::{Key, Value};
use crate::ValueStore;

use super::Store;

pub use rocksdb::DB;

impl<TKey> Store<TKey> for DB
    where
        TKey: Key
{
    fn store_value(&mut self, key: TKey, value: Value) -> Result<(), Error> {
        match self.put(key.as_slice(), value.as_span()) {
            Ok(_) => Ok(()),
            Err(e) => Err(convert_err(e))
        }
    }

    fn get_value(&mut self, key: TKey) -> Option<Value> {
        match self.get_pinned(key.as_slice()) {
            Ok(data) => match data {
                Some(data) => {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let mut cursor = Cursor::new(data);
                    
                    rt.block_on(Value::read_from(&mut cursor)).ok()
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

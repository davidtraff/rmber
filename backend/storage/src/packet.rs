use super::{Error, Key, Value};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub enum Packet<TKey>
where
    TKey: Key,
{
    /// Subscribe to a point.
    Subscribe { token: TKey, id: TKey },
    /// Update a point.
    Update {
        token: TKey,
        id: TKey,
        new_value: Value,
    },
    /// List points.
    List { token: TKey, id: TKey },
}

impl<TKey: Key> Packet<TKey> {
    pub fn write_to<TTarget>(&self, target: &mut TTarget) -> Result<(), Error>
    where
        TTarget: Write,
    {
        target.write_u8(self.into())?;

        match self {
            Packet::Subscribe { token, id } => {
                write_key(target, token)?;
                write_key(target, id)?;
            }
            Packet::Update {
                token,
                id,
                new_value,
            } => {
                write_key(target, token)?;
                write_key(target, id)?;
                new_value.write_to(target)?;
            }
            Packet::List { token, id } => {
                write_key(target, token)?;
                write_key(target, id)?;
            }
        };

        Ok(())
    }

    pub fn read_from<TSource>(source: &mut TSource) -> Result<Self, Error>
    where
        TSource: Read,
    {
        let packet_type = source.read_u8()?;

        match packet_type {
            // Subscribe
            1 => {
                let token = read_key(source)?;
                let id = read_key(source)?;

                Ok(Packet::Subscribe { token, id })
            }
            // Update
            2 => {
                let token = read_key(source)?;
                let id = read_key(source)?;
                let new_value = Value::read_from(source)?;

                Ok(Packet::Update {
                    token,
                    id,
                    new_value,
                })
            }
            // List
            3 => {
                let token = read_key(source)?;
                let id = read_key(source)?;

                Ok(Packet::List { token, id })
            }
            _ => Err(Error::new("Invalid packet-type")),
        }
    }
}

fn write_key<TTarget, TKey>(target: &mut TTarget, key: &TKey) -> Result<(), Error>
where
    TTarget: Write,
    TKey: Key,
{
    let data = key.as_slice();
    let len = data.len() as u8;

    target.write_u8(len)?;
    target.write_all(data)?;

    Ok(())
}

fn read_key<TSource, TKey>(source: &mut TSource) -> Result<TKey, Error>
where
    TSource: Read,
    TKey: Key,
{
    let data = read_len_raw(source)?;

    TKey::from_slice(&data)
}

fn read_len_raw<TSource>(source: &mut TSource) -> Result<Vec<u8>, Error>
where
    TSource: Read,
{
    let len = source.read_u8()?;
    let mut data = vec![0u8; len as usize];

    source.read_exact(&mut data)?;

    Ok(data)
}

impl<T: Key> From<&Packet<T>> for u8 {
    fn from(value: &Packet<T>) -> Self {
        match value {
            Packet::Subscribe { token: _, id: _ } => 1,
            Packet::Update {
                token: _,
                id: _,
                new_value: _,
            } => 2,
            Packet::List { token: _, id: _ } => 3,
        }
    }
}

impl<T: Key> From<Packet<T>> for u8 {
    fn from(value: Packet<T>) -> Self {
        u8::from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StringKey;

    #[test]
    fn write_key_works() {
        let key = StringKey::new("test").unwrap();
        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);

        write_key(&mut cursor, &key).unwrap();

        assert_eq!(&cursor.get_ref()[0..1], &[4]);
        assert_eq!(&cursor.get_ref()[1..5], String::from("test").as_bytes());
    }

    #[test]
    fn read_key_works() {
        let mut data = vec![4];

        for byte in String::from("test").as_bytes() {
            data.push(*byte);
        }

        let mut cursor = std::io::Cursor::new(data);

        let key: StringKey = read_key(&mut cursor).unwrap();

        assert_eq!(key.0, String::from("test"));
    }

    #[test]
    fn serialize_subscribe_packet_works() {
        let packet = Packet::Subscribe {
            token: StringKey::new("token").unwrap(),
            id: StringKey::new("pointid").unwrap(),
        };

        let mut target = std::io::Cursor::new(vec![0u8; 100]);

        packet.write_to(&mut target).unwrap();

        assert_eq!(target.position(), 15);

        assert_eq!(
            &target.get_ref()[0..15],
            &[
                1, // Packet-id,
                5, // token-length
                // _______
                116, //  |
                111, //  |
                107, //  | <-- "token"
                101, //  |
                110, //  |
                // ______|
                7, // id-length
                // _______
                112, //  |
                111, //  |
                105, //  | <-- "pointid"
                110, //  |
                116, //  |
                105, //  |
                100, //  |
                // ______|
            ]
        );
    }

    #[test]
    fn serialize_update_packet_works() {
        let packet = Packet::Update {
            token: StringKey::new("token").unwrap(),
            id: StringKey::new("pointid").unwrap(),
            new_value: Value::I64(1234),
        };

        let mut target = std::io::Cursor::new(vec![0u8; 100]);

        packet.write_to(&mut target).unwrap();

        assert_eq!(target.position(), 24);

        assert_eq!(
            &target.get_ref()[0..24],
            &[
                2, // Packet-id,
                5, // token-length
                // _______
                116, //  |
                111, //  |
                107, //  | <-- "token"
                101, //  |
                110, //  |
                // ______|
                7, // id-length
                // _______
                112, //  |
                111, //  |
                105, //  | <-- "pointid"
                110, //  |
                116, //  |
                105, //  |
                100, //  |
                // ______|
                11, // Value-id
                // _____
                0, //  |
                0, //  |
                0, //  | <-- 1234 in i64
                0, //  |
                0, //  |
                0, //  |
                4, //  |
                210, //|
                // ____|
            ]
        );
    }

    #[test]
    fn deserialize_subscribe_packet() {
        let data = vec![
            1u8, // Packet-id,
            5, // token-length
            // _______
            116, //  |
            111, //  |
            107, //  | <-- "token"
            101, //  |
            110, //  |
            // ______|
            7, // id-length
            // _______
            112, //  |
            111, //  |
            105, //  | <-- "pointid"
            110, //  |
            116, //  |
            105, //  |
            100, //  |
            // ______|
        ];
        let mut data = std::io::Cursor::new(data);
        let packet = Packet::<StringKey>::read_from(&mut data).unwrap();

        assert_eq!(packet, Packet::<StringKey>::Subscribe {
            token: StringKey::new("token").unwrap(),
            id: StringKey::new("pointid").unwrap(),
        });
    }

    #[test]
    fn deserialize_update_packet() {
        let data = vec![
            2u8, // Packet-id,
            5, // token-length
            // _______
            116, //  |
            111, //  |
            107, //  | <-- "token"
            101, //  |
            110, //  |
            // ______|
            7, // id-length
            // _______
            112, //  |
            111, //  |
            105, //  | <-- "pointid"
            110, //  |
            116, //  |
            105, //  |
            100, //  |
            // ______|
            11, // Value-id
            // _____
            0, //  |
            0, //  |
            0, //  | <-- 1234 in i64
            0, //  |
            0, //  |
            0, //  |
            4, //  |
            210, //|
            // ____|
        ];
        let mut data = std::io::Cursor::new(data);
        let packet = Packet::<StringKey>::read_from(&mut data).unwrap();

        assert_eq!(packet, Packet::<StringKey>::Update {
            token: StringKey::new("token").unwrap(),
            id: StringKey::new("pointid").unwrap(),
            new_value: Value::I64(1234),
        });
    }
}

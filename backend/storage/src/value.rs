use super::Error;
use tokio_byteorder::{BigEndian, AsyncReadBytesExt, AsyncWriteBytesExt};
use tokio::io::{AsyncRead, AsyncWrite};
use std::marker::Unpin;

#[derive(Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Blob(Vec<u8>),
    String(String),

    U8(u8),
    I8(i8),

    U16(u16),
    I16(i16),

    U32(u32),
    I32(i32),

    U64(u64),
    I64(i64),

    F32(f32),
    F64(f64),
}

impl Value {
    pub fn as_span(&self) -> &[u8] {
        todo!();
    }

    pub async fn write_to<TTarget>(&self, target: &mut TTarget) -> Result<(), Error>
    where
        TTarget: AsyncWrite + Unpin,
    {
        target.write_u8(self.into()).await?;

        match self {
            Value::Boolean(_) => panic!(),
            Value::Blob(v) => {
                target.write_u32::<BigEndian>(v.len() as u32).await?;
                tokio::io::AsyncWriteExt::write_all(target, &v).await?;
            },
            Value::String(v) => {
                target.write_u32::<BigEndian>(v.len() as u32).await?;
                tokio::io::AsyncWriteExt::write_all(target, v.as_bytes()).await?;
            },
            Value::U8(v) => target.write_u8(*v).await?,
            Value::I8(v) => target.write_i8(*v).await?,

            Value::U16(v) => target.write_u16::<BigEndian>(*v).await?,
            Value::I16(v) => target.write_i16::<BigEndian>(*v).await?,

            Value::U32(v) => target.write_u32::<BigEndian>(*v).await?,
            Value::I32(v) => target.write_i32::<BigEndian>(*v).await?,

            Value::U64(v) => target.write_u64::<BigEndian>(*v).await?,
            Value::I64(v) => target.write_i64::<BigEndian>(*v).await?,

            Value::F32(v) => target.write_f32::<BigEndian>(*v).await?,
            Value::F64(v) => target.write_f64::<BigEndian>(*v).await?,
        };

        Ok(())
    }

    pub async fn read_from<TSource>(source: &mut TSource) -> Result<Self, Error>
    where
        TSource: AsyncRead + Unpin,
    {
        let value_type = source.read_u8().await?;

        let result = match value_type {
            // Boolean
            1 => panic!(),
            // Blob
            2 => {
                let len = source.read_u32::<BigEndian>().await?;
                let mut data = vec![0u8; len as usize];
                tokio::io::AsyncReadExt::read_exact(source, &mut data).await?;

                Value::Blob(data)
            },
            // String
            3 => {
                let len = source.read_u32::<BigEndian>().await?;
                let mut data = vec![0u8; len as usize];
                tokio::io::AsyncReadExt::read_exact(source, &mut data).await?;
                let string = String::from_utf8(data)?;

                Value::String(string)
            },
            // 8
            4 => Value::U8(source.read_u8().await?),
            5 => Value::I8(source.read_i8().await?),
            // 16
            6 => Value::U16(source.read_u16::<BigEndian>().await?),
            7 => Value::I16(source.read_i16::<BigEndian>().await?),
            // 32
            8 => Value::U32(source.read_u32::<BigEndian>().await?),
            9 => Value::I32(source.read_i32::<BigEndian>().await?),
            // 64
            10 => Value::U64(source.read_u64::<BigEndian>().await?),
            11 => Value::I64(source.read_i64::<BigEndian>().await?),
            // Floats
            12 => Value::F32(source.read_f32::<BigEndian>().await?),
            13 => Value::F64(source.read_f64::<BigEndian>().await?),
            _ => unimplemented!(),
        };

        Ok(result)
    }
}

impl From<&Value> for u8 {
    fn from(value: &Value) -> Self {
        match value {
            Value::Boolean(_) => 1,
            Value::Blob(_) => 2,
            Value::String(_) => 3,
            Value::U8(_) => 4,
            Value::I8(_) => 5,
            Value::U16(_) => 6,
            Value::I16(_) => 7,
            Value::U32(_) => 8,
            Value::I32(_) => 9,
            Value::U64(_) => 10,
            Value::I64(_) => 11,
            Value::F32(_) => 12,
            Value::F64(_) => 13,
        }
    }
}

impl From<Value> for u8 {
    fn from(value: Value) -> Self {
        u8::from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_maps_to_correct_values() {
        assert_eq!(u8::from(Value::Boolean(false)), 1);
        assert_eq!(u8::from(Value::Blob(vec![])), 2);
        assert_eq!(u8::from(Value::String(String::new())), 3);

        assert_eq!(u8::from(Value::U8(0)), 4);
        assert_eq!(u8::from(Value::I8(0)), 5);

        assert_eq!(u8::from(Value::U16(0)), 6);
        assert_eq!(u8::from(Value::I16(0)), 7);

        assert_eq!(u8::from(Value::U32(0)), 8);
        assert_eq!(u8::from(Value::I32(0)), 9);

        assert_eq!(u8::from(Value::U64(0)), 10);
        assert_eq!(u8::from(Value::I64(0)), 11);

        assert_eq!(u8::from(Value::F32(0f32)), 12);
        assert_eq!(u8::from(Value::F64(0f64)), 13);
    }

    #[tokio::test]
    async fn serializes_8_correctly() {
        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::U8(u8::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(&cursor.get_ref()[0..2], &[4, 255]);

        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::I8(i8::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(&cursor.get_ref()[0..2], &[5, 127]);
    }

    #[tokio::test]
    async fn serializes_16_correctly() {
        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::U16(u16::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(&cursor.get_ref()[0..3], &[6, 255, 255]);

        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::I16(i16::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(&cursor.get_ref()[0..3], &[7, 127, 255]);
    }

    #[tokio::test]
    async fn serializes_32_correctly() {
        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::U32(u32::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(&cursor.get_ref()[0..5], &[8, 255, 255, 255, 255]);

        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::I32(i32::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(&cursor.get_ref()[0..5], &[9, 127, 255, 255, 255]);
    }

    #[tokio::test]
    async fn serializes_64_correctly() {
        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::U64(u64::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(
            &cursor.get_ref()[0..9],
            &[10, 255, 255, 255, 255, 255, 255, 255, 255]
        );

        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::I64(i64::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(
            &cursor.get_ref()[0..9],
            &[11, 127, 255, 255, 255, 255, 255, 255, 255]
        );
    }

    #[tokio::test]
    async fn serializes_f_correctly() {
        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::F32(f32::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(&cursor.get_ref()[0..5], &[12, 127, 127, 255, 255]);

        let mut cursor = std::io::Cursor::new(vec![0u8; 100]);
        let value = Value::F64(f64::MAX);
        value.write_to(&mut cursor).await.unwrap();
        assert_eq!(
            &cursor.get_ref()[0..9],
            &[13, 127, 239, 255, 255, 255, 255, 255, 255]
        );
    }
}

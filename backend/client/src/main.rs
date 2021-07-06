use protocol::{Packet, StringKey, Value};
use tokio::{net::TcpStream};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    
    let schema = "
        first_namespace {
            - name: string
            - some_value: u8 | u16
        }
    ";

    let schema = Packet::<StringKey>::RegisterSchema { schema: String::from(schema) };
    schema.write_to(&mut stream).await.unwrap();

    let update = Packet::<StringKey>::Update { id: StringKey::new("first_namespace/some_value").unwrap(), new_value: Value::U8(255) };
    update.write_to(&mut stream).await.unwrap();
}

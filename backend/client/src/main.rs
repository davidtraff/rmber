use protocol::{Packet, StringKey, Value};
use tokio::net::TcpStream;

type PCT = Packet<StringKey>;

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();

    let schema = "
        first_namespace {
            - name: string
            - some_value: i32 | u16
        }
    ";

    let schema = PCT::RegisterSchema {
        schema: String::from(schema),
    };
    schema.write_to(&mut stream).await.unwrap();

    assert_eq!(PCT::Ok {}, PCT::read_from(&mut stream).await.unwrap());

    let sub = PCT::Subscribe {
        id: StringKey::new("first_namespace/some_value").unwrap(),
    };
    sub.write_to(&mut stream).await.unwrap();

    assert_eq!(PCT::Ok {}, PCT::read_from(&mut stream).await.unwrap());

    const SIZE: usize = 1000;
    let start = std::time::Instant::now();
    for i in 0..SIZE {
        let update = PCT::Update {
            id: StringKey::new("first_namespace/some_value").unwrap(),
            new_value: Value::I32(i as i32),
        };
        update.write_to(&mut stream).await.unwrap();

        assert_eq!(PCT::Ok {}, PCT::read_from(&mut stream).await.unwrap());
        assert_eq!(
            PCT::Update {
                id: StringKey::new("first_namespace/some_value").unwrap(),
                new_value: Value::I32(i as i32),
            },
            PCT::read_from(&mut stream).await.unwrap()
        );

        // println!("{:?}", ok);
    }

    let diff = std::time::Instant::now() - start;

    println!("{}", diff.as_millis() as f64 / SIZE as f64);
}

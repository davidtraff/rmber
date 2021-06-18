use std::{collections::HashMap, time::{Duration, SystemTime, UNIX_EPOCH}};
use protocol::{Packet, StringKey};
use tokio::{net::TcpStream, time::sleep};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let mut values = vec![];
    loop {
        let packet = Packet::Subscribe {
            id: StringKey::new("test-point").unwrap(),
        };

        let before = time();

        packet.write_to(&mut stream).await.unwrap();
        let response = Packet::<StringKey>::read_from(&mut stream).await.unwrap();

        let now = time();
        let diff = now - before;

        values.push(diff);

        let avg = average(values.as_slice());
        let median = median(values.clone().as_mut_slice());
        let mode = mode(values.as_slice());

        println!("{:?}", response);
        println!("Avg: {}, Median: {}, Mode: {}", avg as u128, median, mode);

        sleep(Duration::from_millis(100)).await;
    }
}

fn average(numbers: &[u128]) -> f32 {
    numbers.iter().sum::<u128>() as f32 / numbers.len() as f32
}

fn median(numbers: &mut [u128]) -> u128 {
    numbers.sort();
    let mid = numbers.len() / 2;
    numbers[mid]
}

fn mode(numbers: &[u128]) -> u128 {
    let mut occurrences = HashMap::new();

    for &value in numbers {
        *occurrences.entry(value).or_insert(0) += 1;
    }

    occurrences
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(val, _)| val)
        .expect("Cannot compute the mode of zero numbers")
}

fn time() -> u128 {
    SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros()
}

use tokio::io;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result {
    let input = read_until_break().await;
    assert_eq!(
        input,
        "-sdf hgt -lat 44.73566 -lon -68.82446 -txh 4 -f 900 -erp 5 -rxh 2 -rt -90 -dbm -m -o test4 -R 2 -res 1200 -pm 4 -rla 44.73436 -rlo -68.81993"
    );
    println!("65.1 -25.9 110.4");
    Ok(())
}

async fn read_until_break() -> String {
    use futures::stream::StreamExt;
    use tokio_util::codec::{FramedRead, LinesCodec};
    let stdin = io::stdin();
    let mut reader = FramedRead::new(stdin, LinesCodec::new());
    reader.next().await.unwrap().unwrap()
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

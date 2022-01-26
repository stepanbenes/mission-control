   
#![warn(rust_2018_idioms)]

use futures::stream::StreamExt;
use std::{env, io, str};
use tokio_util::codec::{Decoder, Encoder, LinesCodec, FramedRead, Framed};

use bytes::BytesMut;
use tokio_serial::SerialPortBuilderExt;
use futures::sink::SinkExt;

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/ttyUSB0";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

// struct LineCodec;

// impl Decoder for LineCodec {
//     type Item = String;
//     type Error = io::Error;

//     fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//         let newline = src.as_ref().iter().position(|b| *b == b'\n');
//         if let Some(n) = newline {
//             let line = src.split_to(n + 1);
//             return match str::from_utf8(line.as_ref()) {
//                 Ok(s) => Ok(Some(s.to_string())),
//                 Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
//             };
//         }
//         Ok(None)
//     }
// }

// impl Encoder<String> for LineCodec {
//     type Error = io::Error;

//     fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
//         Ok(())
//     }
// }

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());

    let port = tokio_serial::new(tty_path, 9600).open_native_async()?;

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let mut io = Framed::new(port, LinesCodec::new()); //LineCodec.framed(port);

    // while let Some(line_result) = io.next().await {
    //     let line = line_result.expect("Failed to read line");
    //     println!("{}", line);
    //     io.send("pi".to_string()).await.expect("Failed to send text");
    // }

    loop {
        tokio::select! {
            Ok(stdin_line) = do_the_line() => {
                io.send(stdin_line).await.expect("Failed to send text");
            },
            Some(Ok(serial_line)) = io.next() => {
                println!("{}", serial_line);
            }
        }
    }

    Ok(())
}

async fn do_the_line() -> Result<String, Box<dyn std::error::Error>> {
    let stdin = tokio::io::stdin();
    let mut reader = FramedRead::new(stdin, LinesCodec::new());
    let line = reader.next().await.transpose()?.unwrap();
    Ok(line)
}
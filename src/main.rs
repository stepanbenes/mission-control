//#![warn(rust_2018_idioms)]

use tokio::signal::{ctrl_c, unix::{signal, SignalKind}};
use futures::stream::StreamExt;
use std::{
    env,
    str,
};
use tokio_util::codec::{Framed, FramedRead, LinesCodec};

use futures::sink::SinkExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use stick::{Controller, Event, Listener};

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

    let mut port = tokio_serial::new(tty_path, 9600).open_native_async()?;

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let mut io = Framed::new(port, LinesCodec::new());

    //let stdin = tokio::io::stdin();
    //let mut reader = FramedRead::new(stdin, LinesCodec::new()); // stdin is blocking and prevents shutdown

    //let mut controllers: Vec<_> = Vec::<Controller>::new();
    
    //let mut listener = Listener::default();

    // TODO: works only for connected gamepad and first three devices, otherwise crashes on assert (incompattible device)
    // TODO: adapt implementation to create controller instance from device address or name
    // see: https://github.com/libcala/stick/blob/ab3bdd7746a19b0319e21a246e3e66bdd4882f70/stick/src/raw/linux.rs#L745
    // see: https://github.com/libcala/stick/blob/ab3bdd7746a19b0319e21a246e3e66bdd4882f70/stick/src/raw/linux.rs#L762
    // see: https://github.com/libcala/stick/blob/ab3bdd7746a19b0319e21a246e3e66bdd4882f70/stick/src/raw/linux.rs#L785
    //controllers.push((&mut listener).await); // gamepad buttons and axes
    //controllers.push((&mut listener).await); // gamepad motion
    //controllers.push((&mut listener).await); // gamepad touchpad

    let mut sigterm_stream = signal(SignalKind::terminate())?;

    loop {
        tokio::select! {
            serial_line = read_serial_line(&mut io) => {
                let line = serial_line.expect("Failed to read line from serial");
                println!("serial: {}", line);
            },
            // stdin_line = read_stdin_line(&mut reader) => {
            //     let line = stdin_line.expect("Failed to read line from stdin");
            //     println!("stdin: {}", line);
            //     io.send(line).await.expect("Failed to send text");
            // },
            _ = ctrl_c() => {
                println!("Received ctrl+c. Shutting down.");
                break;
            },
            _ = sigterm_stream.recv() => {
                println!("Received SIGTERM. Shutting down.");
                break;
            },
            
            // event = &mut controllers[0] => {
            //     println!("{:?}", event);
            //     match event {
            //         Event::Disconnect => {
            //         }
            //         Event::ActionA(pressed) => {
            //             controllers[0].rumble(1.0f32);
            //         }
            //         Event::ActionB(pressed) => {
            //             io.send(format!("{}", pressed)).await.expect("Failed to send text");
            //             //controller.rumble(f32::from(u8::from(pressed)));
            //         }
            //         Event::BumperL(pressed) => {
                        
            //         }
            //         Event::BumperR(pressed) => {

            //         }
            //         _ => {}
            //     }
            // },
        }
        println!("---");
    }

    Ok(())
}

// async fn read_stdin_line(reader: &mut FramedRead<tokio::io::Stdin, LinesCodec>) -> Result<String, Box<dyn std::error::Error>> {
//     Ok(reader.next().await.transpose()?.unwrap())
// }

async fn read_serial_line(
    io: &mut Framed<SerialStream, LinesCodec>,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(io.next().await.unwrap()?)
}

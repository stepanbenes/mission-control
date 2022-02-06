mod stick;

use futures::future::Map;
use futures::stream::FuturesUnordered;
use futures::FutureExt;
use futures::SinkExt;
use tokio::signal::{ctrl_c, unix::{signal, SignalKind}};
use futures::stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use tokio_serial::{SerialPortBuilderExt, SerialStream};

use stick::{Controller, Event, Listener};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut io = open_serial_port("/dev/ttyUSB0")?;

    let mut controllers: Vec<_> = Vec::<Controller>::new();
    
    let mut listener = Listener::default();

    // TODO: works only for connected gamepad and first three devices, otherwise crashes on assert (incompattible device)
    // TODO: adapt implementation to create controller instance from device address or name
    // see: https://github.com/libcala/stick/blob/ab3bdd7746a19b0319e21a246e3e66bdd4882f70/stick/src/raw/linux.rs#L745
    // see: https://github.com/libcala/stick/blob/ab3bdd7746a19b0319e21a246e3e66bdd4882f70/stick/src/raw/linux.rs#L762
    // see: https://github.com/libcala/stick/blob/ab3bdd7746a19b0319e21a246e3e66bdd4882f70/stick/src/raw/linux.rs#L785
    //controllers.push((&mut listener).await); // gamepad buttons and axes
    //controllers.push((&mut listener).await); // gamepad motion
    //controllers.push((&mut listener).await); // gamepad touchpad

    //controllers.push((&mut listener).await); // fails!

    let mut sigterm_stream = signal(SignalKind::terminate())?;

    // let futures = controllers
    // .into_iter()
    // .enumerate()
    // .map(|(i, fut)| fut.map(move |res| (i, res)))
    // .collect::<FuturesUnordered<_>>();

    //let controller_futures = FuturesUnordered::<futures::Map::<(Controller, Controller)>>::new();

    loop {
        let mut controllers_futures = FuturesUnordered::<Controller>::new();
        tokio::select! {
            new_controller = (&mut listener) => {
                println!("New device connected: {} ({})", new_controller.name(), new_controller.id());
                controllers.push(new_controller);
            },
            serial_line = read_serial_line(&mut io) => {
                let line = serial_line.expect("Failed to read line from serial");
                println!("serial: {}", line);
            },
            _ = ctrl_c() => {
                println!("Received ctrl+c. Shutting down.");
                //write_to_serial(&mut io, "huhu").await.expect("Failed to write line to serial");
                break;
            },
            _ = sigterm_stream.recv() => {
                println!("Received SIGTERM. Shutting down.");
                break;
            },
            
            event = controllers_futures.select_next_some() => {
                println!("{:?}", event);
                match event {
                    Event::Disconnect => {
                    }
                    Event::ActionA(pressed) => {
                        //controllers[0].rumble(1.0f32);
                    }
                    Event::ActionB(pressed) => {
                        io.send(format!("{}", pressed)).await.expect("Failed to send text");
                        //controller.rumble(f32::from(u8::from(pressed)));
                    }
                    Event::BumperL(pressed) => {
                        
                    }
                    Event::BumperR(pressed) => {

                    }
                    _ => {}
                }
            },

        }
        println!("---");
    }

    Ok(())
}

fn open_serial_port(tty_path: &str) -> Result<Framed<tokio_serial::SerialStream, tokio_util::codec::LinesCodec>, Box<dyn std::error::Error>> {
    let mut port = tokio_serial::new(tty_path, 9600).open_native_async()?;
    #[cfg(unix)]
    port.set_exclusive(false).expect("Unable to set serial port exclusive to false");
    Ok(Framed::new(port, LinesCodec::new()))
}

async fn read_serial_line(
    io: &mut Framed<SerialStream, LinesCodec>,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(io.next().await.unwrap()?)
}

async fn write_to_serial(io: &mut Framed<SerialStream, LinesCodec>, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(io.send(text).await?)
}

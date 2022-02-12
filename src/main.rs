mod stick;
mod power;

#[macro_use]
extern crate lazy_static;

use std::time::{Instant, Duration};
use std::collections::HashMap;
use futures::stream::FuturesUnordered;
use futures::FutureExt;
use futures::SinkExt;
use tokio::signal::{ctrl_c, unix::{signal, SignalKind}};
use futures::stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use tokio_serial::{SerialPortBuilderExt, SerialStream};

use stick::{Controller, Event, Listener, ControllerProvider};

// ==================================================
// REGEX definitions >>>

const EVENT_FILE_PATTERN: &str = "event[1-9][0-9]*"; // ignore event0

lazy_static! {
    static ref EVENT_FILE_REGEX: regex::Regex = regex::Regex::new(EVENT_FILE_PATTERN).unwrap();
    static ref DEVICE_INFO_ADDRESS_LINE_REGEX: regex::Regex = regex::Regex::new("^U: Uniq=([a-zA-Z0-9:]+)$").unwrap();
    static ref DEVICE_INFO_HANDLERS_LINE_REGEX: regex::Regex = regex::Regex::new("^H: Handlers=([a-zA-Z0-9\\s]+)$").unwrap();
}

// ==================================================

use tokio::sync::mpsc;

async fn gamepad_discovery_loop(tx: mpsc::Sender<String>) {
    let mut listener = Listener::new();
    loop {
        let controller_path = (&mut listener).await;
        tx.send(controller_path).await.expect("Could not send controller path via channel");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<String>(32);
    
    let _handle = std::thread::spawn(move || {
        let runtime2 = tokio::runtime::Runtime::new().expect("Runtime for gamepad discovery loop could not be created");
        runtime2.block_on(gamepad_discovery_loop(tx))
    });

    let runtime1 = tokio::runtime::Runtime::new()?;
    runtime1.block_on(main_loop(rx))
}

async fn main_loop(mut rx: mpsc::Receiver<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut io = open_serial_port("/dev/ttyUSB0")?;

    let mut controllers: Vec<_> = Vec::<Controller>::new();
    let mut recently_disconnected_controllers = HashMap::<String, Instant>::new();
    
    let controller_provider = ControllerProvider::new();

    let mut sigterm_stream = signal(SignalKind::terminate())?;

    loop {

        tokio::select! {
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

            Some(controller_path) = rx.recv() => {
                //println!("Received message '{}'", controller_path);
                if !controllers.iter().any(|c| c.filename() == controller_path) {
                    let was_recently_disconnected = 
                        if let Some(time_disconnected) = recently_disconnected_controllers.get(&controller_path) {
                            if Instant::now() - *time_disconnected < Duration::from_millis(1000) {
                                true
                            }
                            else {
                                recently_disconnected_controllers.remove(&controller_path);
                                false
                            }
                        }
                        else {
                            false
                        };
                    if !was_recently_disconnected {
                        if let Some(controller) = controller_provider.create_controller(controller_path) {
                            if controller.name() == "Wireless Controller" { // TODO: remove this for recieving motion and touchpad
                                println!("Received new controller '{}', ('{}')", controller.name(), controller.filename());
                                controllers.push(controller);
                            }
                        }
                    }
                }
            },
            
            Some((event, controller_index)) = next_event(&mut controllers) => {
                println!("{:?}", event);
                match event {
                    Event::Disconnect(id) => {
                        println!("Controller {:?} disconnected", id);
                        if let Some(filename) = id {
                            controllers.retain(|c| c.filename() != filename);
                            recently_disconnected_controllers.insert(filename, Instant::now());
                        }
                    }
                    Event::ActionA(_pressed) => {
                        println!("{:?}", event);
                        let c = &mut controllers[controller_index];
                        c.rumble(1.0f32);
                    }
                    Event::ActionB(pressed) => {
                        io.send(format!("{}", pressed)).await.expect("Failed to send text");
                        //controller.ruaddaassww432141s4a2w3d1s4able(f32::from(u8::from(pressed)));
                    }
                    Event::MenuL(pressed) => {
                        if pressed {
                            let ctrl = &controllers[controller_index];
                            if let Some(status) = power::check_power_status(ctrl.filename())? {
                                println!("Power status: {}", status);
                            }
                        }
                    }
                    Event::BumperL(_pressed) => {
                        println!("{:?}", event);
                    }
                    Event::BumperR(_pressed) => {
                        println!("{:?}", event);
                    }
                    _ => {}
                }
            },

        }
        //println!("---");
    }

    Ok(())
}

fn open_serial_port(tty_path: &str) -> Result<Framed<tokio_serial::SerialStream, tokio_util::codec::LinesCodec>, Box<dyn std::error::Error>> {
    let mut port = tokio_serial::new(tty_path, 9600).open_native_async()?;
    #[cfg(unix)]
    port.set_exclusive(false).expect("Unable to set serial port exclusive to false");
    Ok(Framed::new(port, LinesCodec::new()))
}

async fn read_serial_line(io: &mut Framed<SerialStream, LinesCodec>,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(io.next().await.unwrap()?)
}

#[allow(dead_code)]
async fn write_to_serial(io: &mut Framed<SerialStream, LinesCodec>, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(io.send(text).await?)
}

async fn next_event(controllers: &mut Vec<Controller>) -> Option<(Event, usize)> {
    if controllers.is_empty() {
        return None;
    }
    let mut controller_futures = controllers
            .iter_mut()
            .enumerate()
            .map(|(i, controller)| controller.map(move |event| (event, i)))
            .collect::<FuturesUnordered<_>>();
    Some(controller_futures.select_next_some().await)
}
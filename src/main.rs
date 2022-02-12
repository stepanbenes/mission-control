mod stick;
mod power;

#[macro_use]
extern crate lazy_static;

use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::cell::RefCell;
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

use pasts::Loop;
use std::task::Poll::{self, Pending, Ready};

use tokio::sync::mpsc;

type Exit = usize;

struct State {
    listener: Listener,
    controllers: Vec<String>,
}

impl State {
    fn connect(&mut self, controller_path: &str) -> bool {
        if !self.controllers.iter().any(|path| path == controller_path) { // check if controller not yet present
            //println!("Connected controller, file: {}", controller_path);
            self.controllers.push(controller_path.to_string());
            true
        }
        else {
            false
        }
    }
}

async fn event_loop(tx: mpsc::Sender<String>) {
    let mut state = State {
        listener: Listener::new(),
        controllers: Vec::new(),
    };

    loop {
        let controller_path = (&mut state.listener).await;
        tx.send(controller_path).await.unwrap();
    }
    // let player_id = Loop::new(&mut state)
    //     .when(|s| &mut s.listener, State::connect)
    //     .poll(|s| &mut s.controllers, State::event)
    //     .await;

    //println!("p{} ended the session", player_id);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut io = open_serial_port("/dev/ttyUSB0")?;

    let controllers: RefCell<Vec<_>> = RefCell::new(Vec::<Controller>::new());
    let mut recently_disconnected_controllers = HashMap::<String, Instant>::new();
    
    let controller_provider = ControllerProvider::new();

    // loop {
    //     let c = (&mut listener).await;
    //     println!("New controller: {} {} ({})", c.name(), c.id(), c.filename());
    // }

    let (tx, mut rx) = mpsc::channel::<String>(32);

    let handle = std::thread::spawn(|| {
        pasts::block_on(event_loop(tx));
    });

    //if let Some(controller) = self.listener.create_controller(controller_path) {

    let mut sigterm_stream = signal(SignalKind::terminate())?;

    loop {

        tokio::select! {
            // new_controller = (&mut listener) => {
            //     println!("New device connected: {} ({})", new_controller.name(), new_controller.filename());
            //     //if new_controller.name() == "Wireless Controller Touchpad" {
            //         controllers.borrow_mut().push(new_controller);
            //     //}
            // },
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
                if !controllers.borrow().iter().any(|c| c.filename() == controller_path) {
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
                                controllers.borrow_mut().push(controller);
                            }
                        }
                    }
                }
            },
            
            Some((event, controller_index)) = next_event(&controllers) => {
                println!("{:?}", event);
                match event {
                    Event::Disconnect(id) => {
                        println!("Controller {:?} disconnected", id);
                        if let Some(filename) = id {
                            controllers.borrow_mut().retain(|c| c.filename() != filename);
                            recently_disconnected_controllers.insert(filename, Instant::now());
                        }
                    }
                    Event::ActionA(_pressed) => {
                        println!("{:?}", event);
                        let c = &mut controllers.borrow_mut()[controller_index];
                        c.rumble(1.0f32);
                    }
                    Event::ActionB(pressed) => {
                        io.send(format!("{}", pressed)).await.expect("Failed to send text");
                        //controller.ruaddaassww432141s4a2w3d1s4able(f32::from(u8::from(pressed)));
                    }
                    Event::MenuL(pressed) => {
                        if pressed {
                            let ctrl = &controllers.borrow()[controller_index];
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

async fn next_event(controllers: &RefCell<Vec<Controller>>) -> Option<(Event, usize)> {
    if controllers.borrow().is_empty() {
        return None;
    }
    let mut controller_list = controllers.borrow_mut();
    let mut controller_futures = controller_list
            .iter_mut()
            .enumerate()
            .map(|(i, controller)| controller.map(move |event| (event, i)))
            .collect::<FuturesUnordered<_>>();
    Some(controller_futures.select_next_some().await)
}
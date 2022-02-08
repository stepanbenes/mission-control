mod stick;

#[macro_use]
extern crate lazy_static;

use std::cell::RefCell;
use futures::stream::FuturesUnordered;
use futures::FutureExt;
use futures::SinkExt;
use tokio::signal::{ctrl_c, unix::{signal, SignalKind}};
use futures::stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use tokio_serial::{SerialPortBuilderExt, SerialStream};

use stick::{Controller, Event, Listener};

// ==================================================

use pasts::Loop;
use std::task::Poll::{self, Pending, Ready};


type Exit = usize;

struct State {
    listener: Listener,
    controllers: Vec<Controller>,
    rumble: (f32, f32),
}

impl State {
    fn connect(&mut self, controller_path: String) -> Poll<Exit> {
        if let Some(controller) = self.listener.create_controller(controller_path) {
            if !self.controllers.iter().any(|c| c.filename() == controller.filename()) { // check if controller not yet present
                println!(
                    "Connected controller, id: {:016X}, name: {}, file: {}",
                    controller.id(),
                    controller.name(),
                    controller.filename(),
                );
                self.controllers.push(controller);
                // TODO: send controller via channel to main thread
            }
        }
        Pending
    }

    fn event(&mut self, id: usize, event: Event) -> Poll<Exit> {
        let player = id + 1;
        //println!("p{}: {}", player, event);
        match event {
            Event::Disconnect => {
                println!("p{}: {}", player, event);
                self.controllers.swap_remove(id);
            }
            Event::MenuR(true) => {
                println!("p{}: {}", player, event);
                return Ready(player);
            },
            Event::ActionA(pressed) => {
                println!("p{}: {}", player, event);
                self.controllers[id].rumble(f32::from(u8::from(pressed)));
            }
            Event::ActionB(pressed) => {
                println!("p{}: {}", player, event);
                self.controllers[id].rumble(0.5 * f32::from(u8::from(pressed)));
            }
            Event::BumperL(pressed) => {
                println!("p{}: {}", player, event);
                self.rumble.0 = f32::from(u8::from(pressed));
                self.controllers[id].rumble(self.rumble);
            }
            Event::BumperR(pressed) => {
                println!("p{}: {}", player, event);
                self.rumble.1 = f32::from(u8::from(pressed));
                self.controllers[id].rumble(self.rumble);
            }
            _ => {}
        }
        Pending
    }
}

async fn event_loop() {
    let mut state = State {
        listener: Listener::default(),
        controllers: Vec::new(),
        rumble: (0.0, 0.0),
    };

    loop {
        let controller_path = (&mut state.listener).await;
        let _ = state.connect(controller_path);
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

    //let controllers: RefCell<Vec<_>> = RefCell::new(Vec::<Controller>::new());
    
    //let mut listener = Listener::default();

    // loop {
    //     let c = (&mut listener).await;
    //     println!("New controller: {} {} ({})", c.name(), c.id(), c.filename());
    // }

    let handle = std::thread::spawn(|| {
        pasts::block_on(event_loop());
    });

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
            
            // Some((event, controller_index)) = next_event(&controllers) => {
            //     println!("{:?}", event);
            //     match event {
            //         Event::Disconnect => {
            //             let disconnected_controller = controllers.borrow_mut().remove(controller_index);
            //             println!("Controller {name} disconnected", name = disconnected_controller.name());
            //         }
            //         Event::ActionA(_pressed) => {
            //             //controllers[0].rumble(1.0f32);
            //         }
            //         Event::ActionB(pressed) => {
            //             io.send(format!("{}", pressed)).await.expect("Failed to send text");
            //             //controller.ruaddaassww432141s4a2w3d1s4able(f32::from(u8::from(pressed)));
            //         }
            //         Event::BumperL(_pressed) => {
                        
            //         }
            //         Event::BumperR(_pressed) => {

            //         }
            //         _ => {}
            //     }
            // },

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
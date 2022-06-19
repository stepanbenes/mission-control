mod command;
mod deep_space_network;
mod drive;
mod error;
mod event_translator;
mod stick;

#[allow(dead_code)]
mod winch;

#[macro_use]
extern crate lazy_static;

use command::{Command, Motor};
use event_translator::EventTranslator;
use futures::{
    stream::{FuturesUnordered, StreamExt},
    FutureExt,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::{
    signal::{
        ctrl_c,
        unix::{signal, SignalKind},
    },
    sync::mpsc,
};
use winch::Winch;

use drive::Drive;
use stick::{check_controller_power, Controller, ControllerProvider, Event, Listener};

// ==================================================
// REGEX definitions >>>

const EVENT_FILE_PATTERN: &str = "event[1-9][0-9]*"; // ignore event0

lazy_static! {
    static ref EVENT_FILE_REGEX: regex::Regex = regex::Regex::new(EVENT_FILE_PATTERN).unwrap();
    static ref DEVICE_INFO_ADDRESS_LINE_REGEX: regex::Regex =
        regex::Regex::new("^U: Uniq=([a-zA-Z0-9:]+)$").unwrap();
    static ref DEVICE_INFO_HANDLERS_LINE_REGEX: regex::Regex =
        regex::Regex::new("^H: Handlers=([a-zA-Z0-9\\s]+)$").unwrap();
}

// ==================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello!");

    // channel for communication from controller discovery loop to main program loop
    let (controller_sender, controller_receiver) = mpsc::channel::<String>(32);

    // controller discovery loop on own thread
    let _handle = std::thread::spawn(move || {
        let runtime2 = tokio::runtime::Runtime::new()
            .expect("Runtime for controller discovery loop could not be created");
        runtime2.block_on(controller_discovery_loop(controller_sender))
    });

    // main program loop
    let runtime1 = tokio::runtime::Runtime::new()?;
    runtime1.block_on(main_program_loop(controller_receiver))
}

async fn controller_discovery_loop(tx: mpsc::Sender<String>) {
    let mut listener = Listener::new();
    loop {
        let controller_path = (&mut listener).await;
        tx.send(controller_path)
            .await
            .expect("Could not send controller path via channel");
    }
}

async fn main_program_loop(
    mut controller_listener: mpsc::Receiver<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut controllers: Vec<_> = Vec::<Controller>::new();
    let mut disconnected_controllers_times = HashMap::<String, Instant>::new();
    let controller_provider = ControllerProvider::new(vec!["Wireless Controller"]);
    let mut sigterm_stream = signal(SignalKind::terminate())?;
    let mut event_translator = EventTranslator::new();

    let mut drive = result_to_option(Drive::initialize(), "Drive initialization");
    let mut winch = result_to_option(Winch::initialize(), "Winch initialization");

    loop {
        tokio::select! {
            _ = ctrl_c() => {
                println!("Received ctrl+c. Shutting down.");
                break; // break the main event loop
            },
            _ = sigterm_stream.recv() => {
                println!("Received SIGTERM. Shutting down.");
                break; // break the main event loop
            },

            Some(controller_path) = controller_listener.recv() => {
                if let Some(controller) = try_create_new_controller(controller_path, &controller_provider, &controllers, &mut disconnected_controllers_times) {
                    println!("Received new controller '{}', ('{}')", controller.name(), controller.filename());
                    controllers.push(controller);
                }
            },

            Some((event, controller)) = next_controller_event(&mut controllers) => {
                //println!("{:?}", event); // do not print each event

                if let Event::Disconnect(id) = event {
                    println!("Controller {:?} disconnected", id);
                    if let Some(filename) = id {
                        controllers.retain(|c| c.filename() != filename);
                        disconnected_controllers_times.insert(filename, Instant::now());
                        // TODO: stop motor, stop winch
                        // TODO: make disconnect event into disconnect command
                    }
                } else {
                    for command in event_translator.translate(event, controller) {
                        distribute_command(command, drive.as_mut(), winch.as_mut(), &mut controllers)?;
                    }
                }
            },

        }
    }

    if let Some(winch) = winch {
        winch.join()?;
    }

    Ok(())
}

fn distribute_command(
    command: Command,
    drive: Option<&mut Drive>,
    winch: Option<&mut Winch>,
    controllers: &mut [Controller],
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Drive { motor, speed } => match motor {
            Motor::Left => {
                if let Some(drive) = drive {
                    drive.left_motor_speed(speed)?;
                }
            }
            Motor::Right => {
                if let Some(drive) = drive {
                    drive.right_motor_speed(speed)?;
                }
            }
            Motor::Winch => {
                if let Some(winch) = winch {
                    winch.wind(speed)?;
                }
            }
        },
        Command::ReleaseWinch => {
            if let Some(winch) = winch {
                winch.release()?;
            }
        }
        Command::CheckGamepadPower(controller_id) => {
            if let Some(power_info) = check_controller_power(&controller_id)? {
                println!("{controller_id}: {power_info}");
            }
        }
        Command::RumbleGamepad(controller_id) => {
            if let Some(controller) = controllers
                .iter_mut()
                .find(|c| c.filename() == controller_id)
            {
                controller.rumble(0.5f32);
            }
        }
        Command::Shutdown => {
            println!("Shutting down...");
            system_shutdown::shutdown()?;
        }
    }
    Ok(())
}

async fn next_controller_event(controllers: &mut [Controller]) -> Option<(Event, &mut Controller)> {
    if controllers.is_empty() {
        return None;
    }
    let (event, controller_index) = {
        let mut controller_futures = controllers
            .iter_mut()
            .enumerate()
            .map(|(i, controller)| controller.map(move |event| (event, i)))
            .collect::<FuturesUnordered<_>>();
        controller_futures.select_next_some().await
    };
    Some((event, &mut controllers[controller_index]))
}

fn try_create_new_controller(
    controller_path: String,
    controller_provider: &ControllerProvider,
    controllers: &[Controller],
    disconnected_controllers_times: &mut HashMap<String, Instant>,
) -> Option<Controller> {
    if !controllers.iter().any(|c| c.filename() == controller_path) {
        let was_recently_disconnected =
            if let Some(time_disconnected) = disconnected_controllers_times.get(&controller_path) {
                if Instant::now() - *time_disconnected < Duration::from_millis(1000) {
                    true
                } else {
                    disconnected_controllers_times.remove(&controller_path);
                    false
                }
            } else {
                false
            };
        if !was_recently_disconnected {
            return controller_provider.create_controller(controller_path);
        }
    }
    None
}

fn result_to_option<T, E: std::fmt::Debug>(result: Result<T, E>, job_name: &str) -> Option<T> {
    match result {
        Ok(value) => {
            Some(value)
        }
        Err(error) => {
            eprintln!("{job_name} failed: {error:?}");
            None
        }
    }
}

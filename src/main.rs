mod command;
mod deep_space_network;
mod drive;
mod drive_dispatcher;
mod error;
mod event_translator;
mod interactive_stdin;
mod pid_controller;
mod stick;

#[allow(dead_code)]
mod winch;

#[macro_use]
extern crate lazy_static;

use command::{Command, Motor};
use deep_space_network::{DeepSpaceNetwork, NetworkMessage};
use drive_dispatcher::DriveDispatcher;
use event_translator::EventTranslator;
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

use interactive_stdin::InteractiveStdin;

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
    let _handle = std::thread::Builder::new()
        .name("controller discovery loop".into())
        .spawn(move || {
            let runtime2 = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Runtime for controller discovery loop could not be created");
            runtime2.block_on(controller_discovery_loop(controller_sender));
            println!("controller discovery thread ended");
        })?;

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
    let mut controller_provider = ControllerProvider::new(vec!["Wireless Controller"]);
    let mut sigterm_stream = signal(SignalKind::terminate())?;
    let mut event_translator = EventTranslator::new();

    let mut winch = result_to_option(Winch::initialize(), "Winch initialization");

    let mut drive_dispatcher = result_to_option(Drive::initialize(), "Drive initialization").map(|drive| DriveDispatcher::new(drive));

    let mut network = result_to_option(
        DeepSpaceNetwork::connect(get_deep_space_hub_url()?).await,
        "Connection to Deep Space Network",
    );

    let mut stdin = InteractiveStdin::new();

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
                if let Some(controller) = controller_provider.try_create_new_controller(controller_path) {
                    println!("Received new controller '{}', ('{}')", controller.name(), controller.filename());
                }
            },
            Some((event, controller)) = controller_provider.next_controller_event() => {
                //println!("{:?}", event);
                for command in event_translator.translate(event, controller) {
                    distribute_command(command, drive_dispatcher.as_mut(), winch.as_mut(), &mut controller_provider)?;
                    // if let Some(network) = network {
                    //     network.call().await?;
                    // }
                }
            },
            Some(message) = next_network_message(&mut network), if network.is_some() => {
                println!("{message:?}");
            },
            Ok(Some(line)) = stdin.next_line() => {
                // TODO: translate line into command
                println!("got line: {line}");
            }
        }
    }

    if let Some(winch) = winch {
        winch.join()?;
    }

    Ok(())
}

async fn next_network_message(
    network_option: &mut Option<DeepSpaceNetwork>,
) -> Option<NetworkMessage> {
    if let Some(network) = network_option {
        match network.listen().await {
            Some(Ok(message)) => {
                return Some(message);
            }
            Some(Err(error)) => {
                eprintln!("Failed to receive Deep Space Network message: {error}");
                // TODO: try to reconnect
                *network_option = None; // network is down, set option to None to indicate that network is not available in the main loop
            }
            None => (),
        }
    }
    None
}

fn get_deep_space_hub_url() -> Result<url::Url, url::ParseError> {
    let server_ip_and_port = "192.168.1.163";
    let hub_name = "deep-space-network";
    let url = url::Url::parse(&format!("ws://{server_ip_and_port}/{hub_name}"))?;
    Ok(url)
}

fn distribute_command(
    command: Command,
    drive_dispatcher: Option<&mut DriveDispatcher>,
    winch: Option<&mut Winch>,
    controller_provider: &mut ControllerProvider,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::HandleGamepadDisconnection(controller_id) => {
            println!("Controller '{controller_id}' disconnected");
            controller_provider.disconnect_controller(controller_id);
            // in case controller disconnected during operation, preventively stop motor, stop winch, stop everything
            if let Some(drive_dispatcher) = drive_dispatcher {
                drive_dispatcher.stop()?;
            }
            if let Some(winch) = winch {
                winch.stop()?;
            }
        }
        Command::Drive { motor, speed } => match motor {
            Motor::Left => {
                if let Some(drive_dispatcher) = drive_dispatcher {
                    drive_dispatcher.set_left_motor_speed(speed)?;
                }
            }
            Motor::Right => {
                if let Some(drive_dispatcher) = drive_dispatcher {
                    drive_dispatcher.set_right_motor_speed(speed)?;
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
            if let Some(controller) = controller_provider.get_mut_controller(&controller_id) {
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

fn result_to_option<T, E: std::fmt::Debug>(result: Result<T, E>, job_name: &str) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            eprintln!("{job_name} failed: {error:?}");
            None
        }
    }
}

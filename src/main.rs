mod command;
mod deep_space_network;
mod string_error;

#[macro_use]
extern crate lazy_static;

use command::{Command, Motor};
use deep_space_network::{DeepSpaceNetwork, NetworkMessage};
use tokio::{
    signal::{
        ctrl_c,
    },
    sync::mpsc,
};

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

    // main program loop
    let runtime1 = tokio::runtime::Runtime::new()?;
    runtime1.block_on(main_program_loop())
}

async fn main_program_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut network = result_to_option(
        DeepSpaceNetwork::connect(get_deep_space_hub_url()?).await,
        "Connection to Deep Space Network",
    );

    loop {
        tokio::select! {
            _ = ctrl_c() => {
                println!("Received ctrl+c. Shutting down.");
                break; // break the main event loop
            },
            Some(message) = next_network_message(&mut network), if network.is_some() => {
                println!("{message:?}");
            }
        }
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
    let server_ip_and_port = "cloudberry:80";
    let hub_name = "deep-space-network";
    let url = url::Url::parse(&format!("ws://{server_ip_and_port}/{hub_name}"))?;
    Ok(url)
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

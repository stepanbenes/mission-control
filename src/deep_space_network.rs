use super::common::*;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use tungstenite::Message;
use url::Url;

pub struct DeepSpaceAntenna {
    socket: tungstenite::WebSocket<std::net::TcpStream>,
}

#[derive(Debug, Serialize, Deserialize)]
enum NetworkMessageType {
    Text,
    MethodInvocation,
    ConnectionEvent,
    MethodReturnValue,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NetworkMessage {
    message_type: NetworkMessageType,
    data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MethodInvocationDescriptor {
    /// The name of the remote method.
    method_name: String,
    /// The arguments passed to the method.
    arguments: Option<Vec<String>>,
    /// The unique identifier of the invocation (Guid).
    identifier: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MethodInvocationResult {
    /// The unique identifier of the invocation (guid).
    identifier: String,
    /// The result of the method call.
    result: Option<String>,
    /// The remote exception of the method call.
    exception: Option<String>,
}

// TODO: improve error handling

impl DeepSpaceAntenna {
    pub fn connect(
        server_ip_and_port: &str,
        hub_name: &str,
    ) -> Result<DeepSpaceAntenna, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(server_ip_and_port)?;
        let (socket, response) = tungstenite::client(
            Url::parse(&format!("ws://{}/{}", server_ip_and_port, hub_name))?,
            stream,
        )
        .expect("Can't connect to Deep space network.");
        println!("Connected to Deep space network.");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            println!("* {}", header);
        }
        socket.get_ref().set_nonblocking(true)?;
        Ok(DeepSpaceAntenna { socket })
    }

    pub fn process_messages(&mut self, _sender: &Sender<Notification>) {
        'message_loop: loop {
            let result = self.socket.read_message();
            match result {
                Ok(Message::Text(text)) => {
                    println!("processing text message: {}", text);
                    let network_message: NetworkMessage = serde_json::from_str(&text).unwrap();
                    match network_message.message_type {
                        NetworkMessageType::MethodInvocation => {
                            let descriptor: MethodInvocationDescriptor =
                                serde_json::from_str(&network_message.data).unwrap();
                            match descriptor.method_name.as_str() {
                                "measure_distance" => {
                                    // TODO: parse arguments
                                    // TODO: call measure_distance
                                    let distance = 42i32;
                                    let result = MethodInvocationResult {
                                        identifier: descriptor.identifier,
                                        result: Some(distance.to_string()),
                                        exception: None,
                                    };
                                    let response = NetworkMessage {
                                        message_type: NetworkMessageType::MethodReturnValue,
                                        data: serde_json::to_string(&result).unwrap(),
                                    };
                                    let response_json = serde_json::to_string(&response).unwrap();
                                    self.socket
                                        .write_message(Message::Text(response_json))
                                        .unwrap();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Ok(message) => {
                    println!("ignoring message: {}", message);
                }
                Err(tungstenite::Error::Io(io_error))
                    if io_error.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    // no message in socket, break the processing loop
                    break 'message_loop;
                }
                Err(fatal_error) => {
                    panic!("{}", fatal_error);
                }
            }
        }
    }
}

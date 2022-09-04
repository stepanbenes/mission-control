pub mod client;
mod data;
pub mod server;

use futures::{stream::StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use super::error::StringError;
pub use data::NetworkMessage;

pub struct DeepSpaceNetwork {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl DeepSpaceNetwork {
    pub async fn connect(
        url: url::Url,
    ) -> Result<Self, tokio_tungstenite::tungstenite::error::Error> {
        let (ws_stream, _) = connect_async(url).await?;

        Ok(Self { ws_stream })
    }

    pub async fn listen(&mut self) -> Option<Result<NetworkMessage, Box<dyn std::error::Error>>> {
        match self.ws_stream.next().await {
            Some(Ok(message)) => Some(DeepSpaceNetwork::parse_message(message)),
            Some(Err(error)) => {
                eprintln!("{error:?}");
                Some(Err(error.into()))
            }
            None => None,
        }
    }

    #[allow(dead_code)]
    pub async fn call(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let network_message = NetworkMessage {
            message_type: data::NetworkMessageType::MethodInvocation,
            data: "{\"methodName\":\"DoMath\",\"arguments\":[4,3]}".to_owned(),
        };
        let message = Message::Text(serde_json::to_string(&network_message)?);
        self.ws_stream.send(message).await?;
        Ok(())
    }

    fn parse_message(message: Message) -> Result<NetworkMessage, Box<dyn std::error::Error>> {
        match message {
            Message::Text(text) => {
                let network_message: NetworkMessage = serde_json::from_str(&text)?;
                Ok(network_message)
            }
            _ => Err(StringError::new("Expected text nessage").into()),
        }
    }
}

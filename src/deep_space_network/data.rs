use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum NetworkMessageType {
    Text = 0,
    MethodInvocation = 1,
    ConnectionEvent = 2,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkMessage {
    pub message_type: NetworkMessageType,
    pub data: String,
}

use gilrs::{Button, Axis, GamepadId};

#[derive(Debug)]
pub enum Notification {
    GamepadButton(Button, GamepadId),
    GamepadAxis(Axis, f32, GamepadId),
    SerialInput(u8),
    //NetworkMessage(String), // TODO: add network communication (use tungstenite)
    //TerminationSignal(i32),
    //ImageTaken { uri: String },
    //DistanceMeasured
    //ArrivedToPosition
}

// #[derive(Debug)]
// pub enum Command {
//     TakeImage,
//     MeasureDistance,
//     GoToPosition
// }

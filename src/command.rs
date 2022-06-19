pub enum Motor {
    Left,
    Right,
    Winch,
}

pub enum Command {
    Drive { motor: Motor, speed: f64 },
    ReleaseWinch,
    CheckGamepadPower(String),
    RumbleGamepad(String),
    Shutdown,
    HandleGamepadDisconnection(String),
}

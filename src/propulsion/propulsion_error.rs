#[derive(Debug)]
pub struct PropulsionError(String);

impl From<rppal::pwm::Error> for PropulsionError {
    fn from(error: rppal::pwm::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<rppal::gpio::Error> for PropulsionError {
    fn from(error: rppal::gpio::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<&str> for PropulsionError {
    fn from(error_message: &str) -> Self {
        Self(error_message.to_string())
    }
}

impl From<String> for PropulsionError {
    fn from(error_message: String) -> Self {
        Self(error_message)
    }
}

impl std::error::Error for PropulsionError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PropulsionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}", self.0)
    }
}
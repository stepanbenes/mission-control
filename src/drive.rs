use rppal::{
    gpio::OutputPin,
    pwm::{Channel, Polarity, Pwm},
};

pub struct Drive {
    forward1_pin: OutputPin,
    backward1_pin: OutputPin,
    forward2_pin: OutputPin,
    backward2_pin: OutputPin,
    pwm0: Pwm,
    pwm1: Pwm,
}

#[derive(Debug)]
pub enum MotorDirection {
    None,
    Forward,
    Backward,
}

impl Drive {
    pub fn initialize() -> Result<Self, PropulsionError> {
        let gpio = rppal::gpio::Gpio::new()?;

        let mut forward1_pin = gpio.get(5)?.into_output();
        let mut backward1_pin = gpio.get(6)?.into_output();
        let mut forward2_pin = gpio.get(26)?.into_output();
        let mut backward2_pin = gpio.get(27)?.into_output();

        forward1_pin.set_low();
        backward1_pin.set_low();

        forward2_pin.set_low();
        backward2_pin.set_low();

        // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 2 Hz with a 25% duty cycle.
        let pwm0 = Pwm::with_frequency(Channel::Pwm0, 75.0, 0.5, Polarity::Normal, false)?;
        let pwm1 = Pwm::with_frequency(Channel::Pwm1, 75.0, 0.5, Polarity::Normal, false)?;

        Ok(Self {
            forward1_pin,
            backward1_pin,
            forward2_pin,
            backward2_pin,
            pwm0,
            pwm1,
        })
    }

    pub fn go(&mut self) -> Result<(), PropulsionError> {
        self.left_motor_direction(MotorDirection::Forward)?;
        self.right_motor_direction(MotorDirection::Forward)?;
        self.pwm0.set_duty_cycle(1.0)?;
        self.pwm1.set_duty_cycle(1.0)?;
        self.pwm0.enable()?;
        self.pwm1.enable()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PropulsionError> {
        self.left_motor_direction(MotorDirection::None)?;
        self.right_motor_direction(MotorDirection::None)?;
        self.pwm0.disable()?;
        self.pwm1.disable()?;
        Ok(())
    }

    fn left_motor_direction(&mut self, direction: MotorDirection) -> Result<(), PropulsionError> {
        match direction {
            MotorDirection::None => {
                self.forward1_pin.set_low();
                self.backward1_pin.set_low();
            }
            MotorDirection::Forward => {
                self.forward1_pin.set_high();
                self.backward1_pin.set_low();
            }
            MotorDirection::Backward => {
                self.forward1_pin.set_low();
                self.backward1_pin.set_high();
            }
        }
        Ok(())
    }

    fn right_motor_direction(&mut self, direction: MotorDirection) -> Result<(), PropulsionError> {
        match direction {
            MotorDirection::None => {
                self.forward2_pin.set_low();
                self.backward2_pin.set_low();
            }
            MotorDirection::Forward => {
                self.forward2_pin.set_high();
                self.backward2_pin.set_low();
            }
            MotorDirection::Backward => {
                self.forward2_pin.set_low();
                self.backward2_pin.set_high();
            }
        }
        Ok(())
    }

    pub fn left_motor_speed(&mut self, speed: f64) -> Result<(), PropulsionError> {
        let duty_cycle;
        if speed == 0.0 {
            duty_cycle = 0_f64;
            self.left_motor_direction(MotorDirection::None)?;
        } else if (-1.0..0.0).contains(&speed) {
            self.left_motor_direction(MotorDirection::Backward)?;
            duty_cycle = Drive::map_from_range_to_range(speed.abs(), 0.0..=1.0, 0.0..=1.0);
        } else if (0.0..=1.0).contains(&speed) {
            self.left_motor_direction(MotorDirection::Forward)?;
            duty_cycle = Drive::map_from_range_to_range(speed, 0.0..=1.0, 0.0..=1.0);
        } else {
            return Err(
                format!("`speed` is outside of allowed range -1..1 (was {}).", speed).into(),
            );
        }
        self.pwm0.set_duty_cycle(duty_cycle)?;
        Ok(self.pwm0.enable()?)
    }

    pub fn right_motor_speed(&mut self, speed: f64) -> Result<(), PropulsionError> {
        let duty_cycle;
        if speed == 0.0 {
            duty_cycle = 0_f64;
            self.right_motor_direction(MotorDirection::None)?;
        } else if (-1.0..0.0).contains(&speed) {
            self.right_motor_direction(MotorDirection::Backward)?;
            duty_cycle = Drive::map_from_range_to_range(speed.abs(), 0.0..=1.0, 0.0..=1.0);
        } else if (0.0..=1.0).contains(&speed) {
            self.right_motor_direction(MotorDirection::Forward)?;
            duty_cycle = Drive::map_from_range_to_range(speed, 0.0..=1.0, 0.0..=1.0);
        } else {
            return Err(
                format!("`speed` is outside of allowed range -1..1 (was {}).", speed).into(),
            );
        }
        self.pwm1.set_duty_cycle(duty_cycle)?;
        Ok(self.pwm1.enable()?)
    }

    fn map_from_range_to_range(
        value: f64,
        from_range: std::ops::RangeInclusive<f64>,
        to_range: std::ops::RangeInclusive<f64>,
    ) -> f64 {
        let from_length = from_range.end() - from_range.start();
        let to_length = to_range.end() - to_range.start();
        let value_normalized = (value - from_range.start()) / from_length;
        to_range.start() + value_normalized * to_length
    }
}

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
        write!(f, "{}", self.0)
    }
}

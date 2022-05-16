use rppal::{pwm::{Channel, Polarity, Pwm}, gpio::OutputPin};
use super::propulsion_error::PropulsionError;

pub struct Drive {
	forward1_pin: OutputPin,
	backward1_pin: OutputPin,
	forward2_pin: OutputPin,
	backward2_pin: OutputPin,
	pwm0: Pwm,
	pwm1: Pwm,
	left_motor_direction: f64,
	right_motor_direction: f64,
	left_motor_speed: f64,
	right_motor_speed: f64,
}

#[derive(Debug)]
pub enum MotorDirection {
	Forward,
	Backward
}

impl Drive {
	pub fn initialize() -> Result<Self, PropulsionError> {
		let gpio = rppal::gpio::Gpio::new()?;

		let mut forward1_pin = gpio.get(22)?.into_output();
		let mut backward1_pin = gpio.get(23)?.into_output();
		let mut forward2_pin = gpio.get(25)?.into_output();
		let mut backward2_pin = gpio.get(24)?.into_output();

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
			left_motor_direction: 1_f64,
    		right_motor_direction: 1_f64,
			left_motor_speed: 0_f64,
			right_motor_speed: 0_f64,
		})
	}

	pub fn go(&mut self) -> Result<(), PropulsionError> {
		self.left_motor_speed = 1_f64;
		self.right_motor_speed = 1_f64;
		self.pwm0.set_duty_cycle(self.left_motor_speed)?;
		self.pwm1.set_duty_cycle(self.right_motor_speed)?;
		self.pwm0.enable()?;
		self.pwm1.enable()?;
		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), PropulsionError> {
		self.left_motor_speed = 0_f64;
		self.right_motor_speed = 0_f64;
		self.pwm0.disable()?;
		self.pwm1.disable()?;
		Ok(())
	}

	pub fn left_motor_direction(&mut self, direction: MotorDirection) -> Result<(), PropulsionError> {
		match direction {
			MotorDirection::Forward => {
				self.left_motor_direction = 1.0_f64;
				self.forward1_pin.set_high();
				self.backward1_pin.set_low();
			}
			MotorDirection::Backward => {
				self.left_motor_direction = -1.0_f64;
				self.forward1_pin.set_low();
				self.backward1_pin.set_high();
			}
		}
		self.pwm0.set_duty_cycle(self.left_motor_speed)?;
		Ok(())
	}

	pub fn right_motor_direction(&mut self, direction: MotorDirection) -> Result<(), PropulsionError> {
		match direction {
			MotorDirection::Forward => {
				self.right_motor_direction = 1.0_f64;
				self.forward2_pin.set_high();
				self.backward2_pin.set_low();
			}
			MotorDirection::Backward => {
				self.right_motor_direction = -1.0_f64;
				self.forward2_pin.set_low();
				self.backward2_pin.set_high();
			}
		}
		self.pwm1.set_duty_cycle(self.right_motor_speed)?;
		Ok(())
	}

	pub fn left_motor_speed(&mut self, speed: f64) -> Result<(), PropulsionError> {
		if speed == 0.0 {
			self.left_motor_speed = 0_f64;
		}
		else if (0.0..=1.0).contains(&speed) {
			// set forward
			self.left_motor_speed = Drive::map_from_range_to_range(speed, 0.0..=1.0, 0.0..=1.0);
		}
		else {
			return Err(format!("`speed` is outside of allowed range -1..1 (was {}).", speed).into());
		}
		self.pwm0.set_duty_cycle(self.left_motor_speed)?;
		Ok(self.pwm0.enable()?)
	}

	pub fn right_motor_speed(&mut self, speed: f64) -> Result<(), PropulsionError> {
		if speed == 0.0 {
			self.right_motor_speed = 0_f64;
		}
		else if (0.0..=1.0).contains(&speed) {
			// set forward
			self.right_motor_speed = Drive::map_from_range_to_range(speed, 0.0..=1.0, 0.5..=1.0);
		}
		else {
			return Err(format!("`speed` is outside of allowed range -1..1 (was {}).", speed).into());
		}
		self.pwm1.set_duty_cycle(self.right_motor_speed)?;
		Ok(self.pwm1.enable()?)
	}

	fn map_from_range_to_range(value: f64, from_range: std::ops::RangeInclusive<f64>, to_range: std::ops::RangeInclusive<f64>) -> f64 {
		let from_length = from_range.end() - from_range.start();
		let to_length = to_range.end() - to_range.start();
		let value_normalized = (value - from_range.start()) / from_length;
		to_range.start() + value_normalized * to_length
	}
}
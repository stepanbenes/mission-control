use rppal::{pwm::{Channel, Polarity, Pwm}, gpio::OutputPin};
use super::propulsion_error::PropulsionError;

pub struct Drive {
	forward1_pin: OutputPin,
	backward1_pin: OutputPin,
	forward2_pin: OutputPin,
	backward2_pin: OutputPin,
	pwm0: Pwm,
	pwm1: Pwm,
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
		})
	}

	#[allow(dead_code)]
	pub fn go_forward(&mut self) -> Result<(), PropulsionError> {
		self.left_motor(Some(1.0))?;
		self.right_motor(Some(1.0))?;
		Ok(())
	}

	#[allow(dead_code)]
	pub fn go_backward(&mut self) -> Result<(), PropulsionError> {
		self.left_motor(Some(-1.0))?;
		self.right_motor(Some(-1.0))?;
		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), PropulsionError> {
		self.left_motor(None)?;
		self.right_motor(None)?;
		Ok(())
	}

	pub fn left_motor(&mut self, velocity: Option<f64>) -> Result<(), PropulsionError> {
		if let Some(velocity) = velocity {
			if velocity == 0.0 {
				self.forward1_pin.set_low();
				self.backward1_pin.set_low();
				self.pwm0.set_duty_cycle(0.0)?;
			}
			else if (-1.0..0.0).contains(&velocity) {
				// set backward
				self.forward1_pin.set_low();
				self.backward1_pin.set_high();
				self.pwm0.set_duty_cycle(Drive::map_from_range_to_range(velocity.abs(), 0.0..=1.0, 0.0..=1.0))?;
			}
			else if (0.0..=1.0).contains(&velocity) {
				// set forward
				self.forward1_pin.set_high();
				self.backward1_pin.set_low();
				self.pwm0.set_duty_cycle(Drive::map_from_range_to_range(velocity, 0.0..=1.0, 0.0..=1.0))?;
			}
			else {
				return Err(format!("`velocity` is outside of allowed range -1..1 (was {}).", velocity).into());
			}
			Ok(self.pwm0.enable()?)
		}
		else {
			Ok(self.pwm0.disable()?)
		}
	}

	pub fn right_motor(&mut self, velocity: Option<f64>) -> Result<(), PropulsionError> {
		if let Some(velocity) = velocity {
			if velocity == 0.0 {
				self.forward2_pin.set_low();
				self.backward2_pin.set_low();
				self.pwm1.set_duty_cycle(0.0)?;
			}
			else if (-1.0..0.0).contains(&velocity) {
				// set backward
				self.forward2_pin.set_low();
				self.backward2_pin.set_high();
				self.pwm1.set_duty_cycle(Drive::map_from_range_to_range(velocity.abs(), 0.0..=1.0, 0.5..=1.0))?;
			}
			else if (0.0..=1.0).contains(&velocity) {
				// set forward
				self.forward2_pin.set_high();
				self.backward2_pin.set_low();
				self.pwm1.set_duty_cycle(Drive::map_from_range_to_range(velocity, 0.0..=1.0, 0.5..=1.0))?;
			}
			else {
				return Err(format!("`velocity` is outside of allowed range -1..1 (was {}).", velocity).into());
			}
			Ok(self.pwm1.enable()?)
		}
		else {
			Ok(self.pwm1.disable()?)
		}
	}

	fn map_from_range_to_range(value: f64, from_range: std::ops::RangeInclusive<f64>, to_range: std::ops::RangeInclusive<f64>) -> f64 {
		let from_length = from_range.end() - from_range.start();
		let to_length = to_range.end() - to_range.start();
		let value_normalized = (value - from_range.start()) / from_length;
		to_range.start() + value_normalized * to_length
	}
}
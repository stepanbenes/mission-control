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
		let pwm0 = Pwm::with_frequency(Channel::Pwm0, 25.0, 0.5, Polarity::Normal, false)?;
		let pwm1 = Pwm::with_frequency(Channel::Pwm1, 25.0, 0.5, Polarity::Normal, false)?;

		Ok(Self {
			forward1_pin,
			backward1_pin,
			forward2_pin,
			backward2_pin,
			pwm0,
			pwm1,
		})
	}

	pub fn go_forward(&mut self) -> Result<(), PropulsionError> {
		self.forward1_pin.set_high();
		self.backward1_pin.set_low();
		self.forward2_pin.set_high();
		self.backward2_pin.set_low();
		self.pwm0.enable()?;
    	self.pwm1.enable()?;
		Ok(())
	}

	pub fn stop(&mut self) -> Result<(), PropulsionError> {
		self.forward1_pin.set_low();
		self.backward1_pin.set_low();
		self.forward2_pin.set_low();
		self.backward2_pin.set_low();
		self.pwm0.disable()?;
    	self.pwm1.disable()?;
		Ok(())
	}
}
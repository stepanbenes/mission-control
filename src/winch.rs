use std::time::Duration;

use rppal::{gpio::{Error, OutputPin}};

/// see: https://ben.akrin.com/driving-a-28byj-48-stepper-motor-uln2003-driver-with-a-raspberry-pi/
/// https://tutorials-raspberrypi.com/how-to-control-a-stepper-motor-with-raspberry-pi-and-l293d-uln2003a/

const STEP_SEQUENCE: [(bool, bool, bool, bool); 8] = [
	(true ,false,false,true ),
	(true ,false,false,false),
	(true ,true ,false,false),
	(false,true ,false,false),
	(false,true ,true ,false),
	(false,false,true ,false),
	(false,false,true ,true ),
	(false,false,false,true )];

const STEP_SLEEP: Duration = Duration::from_millis(2); // careful lowering this, at some point you run into the mechanical limitation of how quick your motor can move

const STEP_COUNT: u32 = 4096; // 5.625*(1/64) per step, 4096 steps is 360°

pub struct Winch {
	in1: OutputPin,
	in2: OutputPin,
	in3: OutputPin,
	in4: OutputPin,
	electromagnet: OutputPin,
}

impl Winch {
	pub fn initialize() -> Result<Self, Error> {
		let gpio = rppal::gpio::Gpio::new()?;
		let in1 = gpio.get(22)?.into_output_low();
		let in2 = gpio.get(23)?.into_output_low();
		let in3 = gpio.get(24)?.into_output_low();
		let in4 = gpio.get(25)?.into_output_low();
		let electromagnet = gpio.get(4)?.into_output_low();
		Ok(Self {
			in1, in2, in3, in4, electromagnet
		})
	}

	pub fn wind(&mut self) {
		for _ in 0..STEP_COUNT {
			self.step_forward(STEP_SLEEP);
		}
	}

	pub fn unwind(&mut self) {
		for _ in 0..STEP_COUNT {
			self.step_backward(STEP_SLEEP);
		}
	}

	pub fn release(&mut self) {
		self.electromagnet.set_high();
		std::thread::sleep(Duration::from_millis(200));
		self.electromagnet.set_low();
	}

	fn step_forward(&mut self, delay: Duration) {
		for state in STEP_SEQUENCE {
			self.set_stepper_motor_pins(state);
			std::thread::sleep(delay);
		}
	}

	#[allow(dead_code)]
	fn step_backward(&mut self, delay: Duration) {
		for state in STEP_SEQUENCE.iter().rev() {
			self.set_stepper_motor_pins(*state);
			std::thread::sleep(delay);
		}
	}

	fn set_stepper_motor_pins(&mut self, (in1_enabled, in2_enabled, in3_enabled, in4_enabled): (bool, bool, bool, bool)) {
		if in1_enabled {
			self.in1.set_high();
		}
		else {
			self.in1.set_low();
		}
		if in2_enabled {
			self.in2.set_high();
		}
		else {
			self.in2.set_low();
		}
		if in3_enabled {
			self.in3.set_high();
		}
		else {
			self.in3.set_low();
		}
		if in4_enabled {
			self.in4.set_high();
		}
		else {
			self.in4.set_low();
		}
	}
}
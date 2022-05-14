use rppal::{gpio::{Trigger, Error, InputPin}};

pub struct HardwareInput {
	button_pin: InputPin,
}

impl HardwareInput {
	pub fn initialize() -> Result<Self, Error> {
		let gpio = rppal::gpio::Gpio::new()?;

		let mut button_pin = gpio.get(4)?.into_input_pullup();
		button_pin.set_async_interrupt(Trigger::FallingEdge, |_| println!("Button pressed!"))?;

		Ok(Self { button_pin })
	}
}
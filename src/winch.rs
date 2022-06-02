use std::{time::Duration, thread::JoinHandle};

use rppal::gpio::{Error, OutputPin};

/// see: https://ben.akrin.com/driving-a-28byj-48-stepper-motor-uln2003-driver-with-a-raspberry-pi/
/// https://tutorials-raspberrypi.com/how-to-control-a-stepper-motor-with-raspberry-pi-and-l293d-uln2003a/

const STEP_SEQUENCE: [(bool, bool, bool, bool); 8] = [
    (true, false, false, true),
    (true, false, false, false),
    (true, true, false, false),
    (false, true, false, false),
    (false, true, true, false),
    (false, false, true, false),
    (false, false, true, true),
    (false, false, false, true),
];

const STEP_SLEEP: Duration = Duration::from_micros(1000); // careful lowering this, at some point you run into the mechanical limitation of how quick your motor can move

const STEP_COUNT: u32 = 512; // 4096 substeps is 360 degrees

pub struct Winch {
	thread_handle: JoinHandle<()>,
	sender: std::sync::mpsc::Sender<WinchCommand>,
}

struct WinchDriver {
    in1: OutputPin,
    in2: OutputPin,
    in3: OutputPin,
    in4: OutputPin,
    electromagnet: OutputPin,
}

impl WinchDriver {
	fn initialize() -> Result<Self, Error> {
		let gpio = rppal::gpio::Gpio::new()?;
		let in1 = gpio.get(22)?.into_output_low();
		let in2 = gpio.get(23)?.into_output_low();
		let in3 = gpio.get(24)?.into_output_low();
		let in4 = gpio.get(25)?.into_output_low();
		let electromagnet = gpio.get(4)?.into_output_high(); // high is off
		let winch_driver = WinchDriver {
			in1,
			in2,
			in3,
			in4,
			electromagnet,
		};
		Ok(winch_driver)
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
        self.electromagnet.set_low();
        std::thread::sleep(Duration::from_millis(200));
        self.electromagnet.set_high();
    }

    fn step_forward(&mut self, delay: Duration) {
        for state in STEP_SEQUENCE {
            self.set_stepper_motor_pins(state);
            std::thread::sleep(delay);
        }
        self.turn_off_motor();
    }

    #[allow(dead_code)]
    fn step_backward(&mut self, delay: Duration) {
        for state in STEP_SEQUENCE.iter().rev() {
            self.set_stepper_motor_pins(*state);
            std::thread::sleep(delay);
        }
        self.turn_off_motor();
    }

    fn turn_off_motor(&mut self) {
        self.set_stepper_motor_pins((false, false, false, false));
    }

    fn set_stepper_motor_pins(
        &mut self,
        (in1_enabled, in2_enabled, in3_enabled, in4_enabled): (bool, bool, bool, bool),
    ) {
        if in1_enabled {
            self.in1.set_high();
        } else {
            self.in1.set_low();
        }
        if in2_enabled {
            self.in2.set_high();
        } else {
            self.in2.set_low();
        }
        if in3_enabled {
            self.in3.set_high();
        } else {
            self.in3.set_low();
        }
        if in4_enabled {
            self.in4.set_high();
        } else {
            self.in4.set_low();
        }
    }
}

enum WinchCommand {
    Wind { speed: f64 },
    Unwind { speed: f64 },
    Stop,
    Release,
}

impl Winch {
    pub fn initialize() -> Result<Self, Error> {
		let (tx, rx) = std::sync::mpsc::channel::<WinchCommand>();

        let thread_handle = std::thread::spawn(move || {
            let mut driver = WinchDriver::initialize().unwrap();
			while let Ok(command) = rx.recv() {
				match command {
                    WinchCommand::Wind { speed: _ } => {
                        // TODO: use rx.try_recv in loop and run winch motor until WinchCommand::Stop is received
                    }
                    WinchCommand::Unwind { speed: _ } => {
                        // TODO: same as above
                    }
                    WinchCommand::Stop => {

                    }
                    WinchCommand::Release => {
                        driver.release();
                    }
                }
			}
        });


        Ok(Self {
			thread_handle,
			sender: tx,
		})
    }

	pub fn wind(&mut self, speed: f64) -> Result<(), WinchError> {
		self.sender.send(WinchCommand::Wind { speed })?;
        Ok(())
    }

    pub fn unwind(&mut self, speed: f64) -> Result<(), WinchError> {
        self.sender.send(WinchCommand::Unwind { speed })?;
        Ok(())
    }
	
    pub fn stop(&mut self) -> Result<(), WinchError> {
        self.sender.send(WinchCommand::Stop)?;
        Ok(())
    }

	pub fn release(&mut self) -> Result<(), WinchError> {
        self.sender.send(WinchCommand::Release)?;
        Ok(())
    }
    
}

#[derive(Debug)]
pub struct WinchError(String);

impl<T> From<std::sync::mpsc::SendError<T>> for WinchError {
    fn from(error: std::sync::mpsc::SendError<T>) -> Self {
        Self(error.to_string())
    }
}

impl std::fmt::Display for WinchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}", self.0)
    }
}

impl std::error::Error for WinchError {
    fn description(&self) -> &str {
        &self.0
    }
}
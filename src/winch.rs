use rppal::gpio::{Error, OutputPin};
use std::{sync::mpsc::TryRecvError, thread::JoinHandle, time::Duration};

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

const MIN_STEP_SLEEP: Duration = Duration::from_micros(800); // careful lowering this, at some point you run into the mechanical limitation of how quick your motor can move
const MAX_STEP_SLEEP: Duration = Duration::from_micros(5000);

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
    }

    #[allow(dead_code)]
    fn step_backward(&mut self, delay: Duration) {
        for state in STEP_SEQUENCE.iter().rev() {
            self.set_stepper_motor_pins(*state);
            std::thread::sleep(delay);
        }
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

impl Drop for WinchDriver {
    fn drop(&mut self) {
        self.turn_off_motor();
        println!("Dropping winch driver.");
    }
}

#[derive(Debug)]
enum WinchCommand {
    Wind { speed: f64 },
    Stop,
    Release,
    Quit,
}

impl Winch {
    pub fn initialize() -> Result<Self, Error> {
        let (tx, rx) = std::sync::mpsc::channel::<WinchCommand>();
        let mut winch_driver = WinchDriver::initialize()?;

        let winch_driver_loop = move || {
            let mut peek_command;
            'outer_loop: while let Ok(command) = rx.recv() {
                peek_command = Some(command);
                'middle_loop: while let Some(command) = peek_command.take() {
                    match command {
                        WinchCommand::Wind { speed } => {
                            'inner_loop: loop {
                                if speed > 0.0 {
                                    winch_driver.step_forward(Winch::map_speed_to_delay(speed));
                                } else if speed < 0.0 {
                                    winch_driver.step_backward(Winch::map_speed_to_delay(speed));
                                }
                                // break the inner loop if there is some command in the queue (except Release command)
                                match rx.try_recv() {
                                    Ok(WinchCommand::Release) => {
                                        winch_driver.release();
                                        continue 'inner_loop;
                                    }
                                    Ok(new_command) => {
                                        match Winch::try_get_last_command(&rx) {
                                            Ok(last_command) => {
                                                peek_command = Some(last_command);
                                            }
                                            Err(TryRecvError::Empty) => {
                                                peek_command = Some(new_command);
                                            }
                                            Err(TryRecvError::Disconnected) => {
                                                break 'outer_loop;
                                            }
                                        }
                                        continue 'middle_loop;
                                    }
                                    Err(TryRecvError::Empty) => {
                                        continue 'inner_loop;
                                    }
                                    Err(TryRecvError::Disconnected) => {
                                        break 'outer_loop;
                                    }
                                }
                            }
                        }
                        WinchCommand::Stop => {
                            winch_driver.turn_off_motor();
                        }
                        WinchCommand::Release => {
                            winch_driver.release();
                        }
                        WinchCommand::Quit => {
                            break 'outer_loop; // quit loop, terminate thread
                        }
                    }
                }
            }
        };

        let thread_handle = std::thread::Builder::new()
            .name("winch thread".into())
            .spawn(winch_driver_loop)?;

        Ok(Self {
            thread_handle,
            sender: tx,
        })
    }

    fn map_speed_to_delay(speed: f64) -> Duration {
        let speed = speed.abs().max(0.0).min(1.0);
        let multiplier = 1.0 - speed;
        Duration::from_secs_f64(
            MIN_STEP_SLEEP.as_secs_f64()
                + (MAX_STEP_SLEEP.as_secs_f64() - MIN_STEP_SLEEP.as_secs_f64()) * multiplier,
        )
    }

    fn try_get_last_command(
        rx: &std::sync::mpsc::Receiver<WinchCommand>,
    ) -> Result<WinchCommand, TryRecvError> {
        let mut last_result = Err(TryRecvError::Empty);
        loop {
            let result = rx.try_recv();
            match result {
                Ok(_) => {
                    last_result = result;
                }
                Err(TryRecvError::Disconnected) => {
                    return result;
                }
                Err(TryRecvError::Empty) => {
                    return last_result;
                }
            }
        }
    }

    pub fn wind(&mut self, speed: f64) -> Result<(), WinchError> {
        if speed == 0.0 {
            self.sender.send(WinchCommand::Stop)?;
        } else {
            self.sender.send(WinchCommand::Wind { speed })?;
        }
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

    pub fn join(self) -> Result<(), Box<dyn std::error::Error>> {
        self.sender.send(WinchCommand::Quit)?;
        println!("Quit command has been sent to winch driver.");
        let result = self
            .thread_handle
            .join()
            .expect("Winch thread could not be joined.");
        Ok(result)
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
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for WinchError {
    fn description(&self) -> &str {
        &self.0
    }
}

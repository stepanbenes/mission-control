use rppal::gpio::{Error, InputPin, Level, OutputPin, Trigger};
use std::time::Duration;
use std::{
    sync::{Arc, Mutex},
    thread,
};

pub struct Sonar {
    trigger_pin: OutputPin,
    echo_pin: InputPin,
}

struct Pulse {
    start: Option<std::time::Instant>,
    end: Option<std::time::Instant>,
}

impl Pulse {
    fn empty() -> Self {
        Pulse {
            start: None,
            end: None,
        }
    }

    fn length(&self) -> Option<std::time::Duration> {
        match (self.start, self.end) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }
}

impl Sonar {
    pub fn initialize() -> Result<Self, Error> {
        let gpio = rppal::gpio::Gpio::new()?;

        let mut trigger_pin = gpio.get(5)?.into_output();
        let echo_pin = gpio.get(6)?.into_input();

        trigger_pin.set_low();

        Ok(Self {
            trigger_pin,
            echo_pin,
        })
    }

    pub fn measure_distance(&mut self) -> Result<f64, Error> {
        // let gpio = rppal::gpio::Gpio::new()?;

        // let mut trigger_pin = gpio.get(5)?.into_output();
        // let mut echo_pin = gpio.get(6)?.into_input();

        self.trigger_pin.set_low();

        thread::sleep(Duration::from_secs(1));

        println!("init echo is_high: {}", self.echo_pin.is_high());

        let pulse: Arc<Mutex<Pulse>> = Arc::new(Mutex::new(Pulse::empty()));
        let pulse_cloned = pulse.clone();
        self.echo_pin.set_async_interrupt(Trigger::Both, move |level| {
            let instant = std::time::Instant::now();
            if level == Level::High {
                println!("echo raising: {:?}", instant);
                pulse_cloned.lock().unwrap().start = Some(instant);
            } else if level == Level::Low {
                println!("echo falling: {:?}", instant);
                pulse_cloned.lock().unwrap().end = Some(instant);
            }
        })?;

        // measure distance
        self.trigger_pin.set_high();
        thread::sleep(Duration::from_micros(10));
        self.trigger_pin.set_low();

        thread::sleep(Duration::from_secs(1));

        self.echo_pin.clear_async_interrupt()?;

        let pulse_length = pulse.lock().unwrap().length();

        if let Some(duration) = pulse_length {
            let distance = duration.as_secs_f64() * 17150_f64;
            Ok(distance)
        } else {
            Ok(0.0)
        }
    }
}

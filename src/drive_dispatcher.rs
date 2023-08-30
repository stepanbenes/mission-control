use crate::drive::PropulsionError;

use super::drive::Drive;

use super::easing::Easing;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::{self, Duration};

pub struct DriveDispatcher {
    drive: Option<Drive>,

    left_motor_speed: f64,
    right_motor_speed: f64,

    left_motor_speed_easing: Easing,
    right_motor_speed_easing: Easing,

    interval: tokio::time::Interval,
    use_interval_ticking: bool,

    easing_trigger: UnboundedSender<()>,
    easing_trigger_handler: UnboundedReceiver<()>,
}

impl DriveDispatcher {
    pub fn new(drive: Option<Drive>) -> DriveDispatcher {
        let time_step = 0.1; // duration of one tick in seconds
        let duration_in_unit_value = 1.0; // 1.0 duration in seconds per 1.0 value distance = 0.5 / 1.0 = 0.5
        let (easing_trigger, easing_trigger_handler) = unbounded_channel::<()>();
        DriveDispatcher {
            drive,
            left_motor_speed: 0.0,
            right_motor_speed: 0.0,
            left_motor_speed_easing: Easing::new(0.0, 0.0, duration_in_unit_value, time_step),
            right_motor_speed_easing: Easing::new(0.0, 0.0, duration_in_unit_value, time_step),
            interval: time::interval(Duration::from_secs_f64(time_step)),
            use_interval_ticking: false,
            easing_trigger,
            easing_trigger_handler,
        }
    }

    pub fn set_left_motor_speed(&mut self, speed: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.left_motor_speed_easing = self
            .left_motor_speed_easing
            .with(self.left_motor_speed, speed);
        self.easing_trigger.send(())?;
        Ok(())
    }

    pub fn set_right_motor_speed(&mut self, speed: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.right_motor_speed_easing = self
            .right_motor_speed_easing
            .with(self.right_motor_speed, speed);
        self.easing_trigger.send(())?;
        Ok(())
    }

    pub fn stop_immediately(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(drive) = self.drive.as_mut() {
            drive.stop()?;
        }

        self.left_motor_speed = 0.0;
        self.right_motor_speed = 0.0;
        self.set_left_motor_speed(0.0)?;
        self.set_right_motor_speed(0.0)?;

        Ok(())
    }

    pub async fn update(&mut self) -> Result<(), PropulsionError> {
        if self.use_interval_ticking {
            self.interval.tick().await; // wait for next tick
        } else {
            self.easing_trigger_handler.recv().await; // wait for next speed command
            self.interval.reset();
            self.use_interval_ticking = true;
        }

        self.left_motor_speed = self.left_motor_speed_easing.apply_easing();
        self.right_motor_speed = self.right_motor_speed_easing.apply_easing();
        if let Some(drive) = self.drive.as_mut() {
            drive.left_motor_speed(self.left_motor_speed)?;
            drive.right_motor_speed(self.right_motor_speed)?;
        }

        if self.left_motor_speed_easing.is_finished() && self.right_motor_speed_easing.is_finished()
        {
            self.use_interval_ticking = false; // replace interval ticking with waiting on changing motor speed by calling either of methods set_left_motor_speed, set_right_motor_speed or stop_immediately
        }

        println!("[{}, {}]", self.left_motor_speed, self.right_motor_speed);
        Ok(())
    }
}

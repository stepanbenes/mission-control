use crate::drive::PropulsionError;

use super::drive::Drive;

use super::easing::Easing;

use tokio::time::{self, Duration};

pub struct DriveDispatcher {
    drive: Option<Drive>,

    left_motor_speed: f64,
    right_motor_speed: f64,

    left_motor_speed_easing: Easing,
    right_motor_speed_easing: Easing,

    interval: tokio::time::Interval,
}

impl DriveDispatcher {
    pub fn new(drive: Option<Drive>) -> DriveDispatcher {
        let time_step = 0.01; // duration of one tick in seconds
        let duration_in_unit_value = 1.0; // 1.0 duration in seconds per 1.0 value distance = 1.0 / 1.0 = 1.0
        DriveDispatcher {
            drive,
            left_motor_speed: 0.0,
            right_motor_speed: 0.0,
            left_motor_speed_easing: Easing::new(0.0, 0.0, duration_in_unit_value, time_step),
            right_motor_speed_easing: Easing::new(0.0, 0.0, duration_in_unit_value, time_step),
            interval: time::interval(Duration::from_secs_f64(time_step)),
        }
    }

    pub fn set_left_motor_speed(&mut self, speed: f64) {
        self.left_motor_speed_easing = self
            .left_motor_speed_easing
            .with(self.left_motor_speed, speed);
    }

    pub fn set_right_motor_speed(&mut self, speed: f64) {
        self.right_motor_speed_easing = self
            .right_motor_speed_easing
            .with(self.right_motor_speed, speed);
    }

    pub fn stop_immediately(&mut self) -> Result<(), PropulsionError> {
        if let Some(drive) = self.drive.as_mut() {
            drive.stop()?;
        }

        self.left_motor_speed = 0.0;
        self.right_motor_speed = 0.0;
        self.set_left_motor_speed(0.0);
        self.set_right_motor_speed(0.0);

        Ok(())
    }

    pub async fn update(&mut self) -> Result<(), PropulsionError> {
        // TODO: if animation not running, await until animation restarts

        // wait interval
        self.interval.tick().await;

        self.left_motor_speed = self.left_motor_speed_easing.apply_easing();
        self.right_motor_speed = self.right_motor_speed_easing.apply_easing();
        if let Some(drive) = self.drive.as_mut() {
            drive.left_motor_speed(self.left_motor_speed)?;
            drive.right_motor_speed(self.right_motor_speed)?;
        }

        println!("[{}, {}]", self.left_motor_speed, self.right_motor_speed);
        Ok(())
    }
}

use crate::drive::PropulsionError;

use super::drive::Drive;
use super::pid_controller::PIDController;

use tokio::time::{self, Duration};

pub struct DriveDispatcher {
    drive: Option<Drive>,

    left_motor_speed: f64,
    right_motor_speed: f64,

    left_motor_pid_controller: PIDController,
    right_motor_pid_controller: PIDController,

    interval: tokio::time::Interval,
}

impl DriveDispatcher {
    pub fn new(drive: Option<Drive>) -> DriveDispatcher {
        DriveDispatcher {
            drive,
            left_motor_speed: 0.0,
            right_motor_speed: 0.0,
            left_motor_pid_controller: PIDController::default(),
            right_motor_pid_controller: PIDController::default(),
            interval: time::interval(Duration::from_millis(500)),
        }
    }

    pub fn set_left_motor_speed(&mut self, speed: f64) {
        self.left_motor_pid_controller.set_setpoint(speed);
    }

    pub fn set_right_motor_speed(&mut self, speed: f64) {
        self.right_motor_pid_controller.set_setpoint(speed);
    }

    pub fn stop(&mut self) -> Result<(), PropulsionError> {
        if let Some(drive) = self.drive.as_mut() {
            drive.stop()?;
        }
        Ok(())
    }

    pub async fn update(&mut self) -> Result<(), PropulsionError> {
        // wait interval
        self.interval.tick().await;
        // left motor
        self.left_motor_speed = self
            .left_motor_pid_controller
            .compute(self.left_motor_speed);
        if let Some(drive) = self.drive.as_mut() {
            drive.left_motor_speed(self.left_motor_speed)?;
        }
        // right motor
        self.right_motor_speed = self
            .right_motor_pid_controller
            .compute(self.right_motor_speed);
        if let Some(drive) = self.drive.as_mut() {
            drive.right_motor_speed(self.left_motor_speed)?;
        }
        println!("[{}, {}]", self.left_motor_speed, self.right_motor_speed);
        Ok(())
    }
}

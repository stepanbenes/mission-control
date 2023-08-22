use crate::drive::PropulsionError;

use super::drive::Drive;
use super::pid_controller::PIDController;

pub struct DriveDispatcher {
    drive: Drive,

    left_motor_pid_controller: PIDController,
    right_motor_pid_controller: PIDController,
}

impl DriveDispatcher {
    pub fn new(drive: Drive) -> DriveDispatcher {
        DriveDispatcher {
            drive,
            left_motor_pid_controller: PIDController::default(),
            right_motor_pid_controller: PIDController::default(),
        }
    }

    pub fn set_left_motor_speed(&mut self, speed: f64) -> Result<(), PropulsionError> {
        self.drive.left_motor_speed(speed)?;
        Ok(())
    }

    pub fn set_right_motor_speed(&mut self, speed: f64) -> Result<(), PropulsionError> {
        self.drive.right_motor_speed(speed)?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PropulsionError> {
        self.drive.stop()?;
        Ok(())
    }
}

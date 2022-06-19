use crate::stick::{Controller, Event};

use crate::command::{Command, Motor};

pub struct EventTranslator {
    menu_left_pressed: bool,
    menu_right_pressed: bool,
}

impl EventTranslator {
    pub fn new() -> Self {
        Self {
            menu_left_pressed: false,
            menu_right_pressed: false,
        }
    }

    pub fn translate(&mut self, event: Event, controller: &Controller) -> Vec<Command> {
        match event {
            Event::Disconnect(Some(controller_id)) => {
                return vec![Command::HandleGamepadDisconnection(controller_id)];
            }
            Event::MenuL(pressed) => {
                self.menu_left_pressed = pressed;
                return self.check_shutdown();
            }
            Event::MenuR(pressed) => {
                self.menu_right_pressed = pressed;
                return self.check_shutdown();
            }
            Event::ActionA(pressed) => {
                if pressed {
                    return vec![
                        Command::Drive {
                            motor: Motor::Left,
                            speed: 0.0,
                        },
                        Command::Drive {
                            motor: Motor::Right,
                            speed: 0.0,
                        },
                    ];
                }
            }
            Event::ActionB(pressed) => {
                if pressed {
                    return vec![Command::RumbleGamepad(controller.filename().to_owned())];
                }
            }
            Event::ActionH(pressed) => {
                if pressed {
                    return vec![Command::ReleaseWinch];
                }
            }
            Event::ActionV(pressed) => {
                if pressed {
                    return vec![
                        Command::Drive {
                            motor: Motor::Left,
                            speed: 1.0,
                        },
                        Command::Drive {
                            motor: Motor::Right,
                            speed: 1.0,
                        },
                    ];
                }
            }
            Event::Exit(pressed) => {
                if pressed {
                    return vec![Command::CheckGamepadPower(controller.filename().to_owned())];
                }
            }
            Event::JoyY(value) => {
                return vec![Command::Drive {
                    motor: Motor::Right,
                    speed: -value,
                }];
            }
            Event::CamY(value) => {
                return vec![Command::Drive {
                    motor: Motor::Left,
                    speed: -value,
                }];
            }
            Event::JoyZ(value) => {
                let speed = (value + 1.0) / 2.0;
                return vec![Command::Drive {
                    motor: Motor::Winch,
                    speed,
                }];
            }
            Event::CamZ(value) => {
                let speed = (value + 1.0) / 2.0;
                return vec![Command::Drive {
                    motor: Motor::Winch,
                    speed: -speed,
                }];
            }
            _ => {}
        }
        vec![]
    }

    fn check_shutdown(&self) -> Vec<Command> {
        if self.menu_left_pressed && self.menu_right_pressed {
            vec![Command::Shutdown]
        } else {
            vec![]
        }
    }
}

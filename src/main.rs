extern crate joydev;
mod serial_port;
use joydev::{event_codes::AbsoluteAxis, event_codes::Key, Device, DeviceEvent, GenericEvent};
use serial_port::*;
use std::sync::{mpsc::channel, Arc};
use std::thread;

enum Notification {
    ControllerButton(joydev::ButtonEvent),
    ControllerAxis(joydev::AxisEvent),
    SerialInput(u8),
    //NetworkCommand(String),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // open serial port
    let serial_port = Arc::new(SerialPort::open("/dev/ttyACM0")?);
    // open joistick controller
    let device = Device::open("/dev/input/js0")?;

    // create communication channel
    let (tx, rx) = channel::<Notification>();

    // read serial port
    {
        let tx = tx.clone();
        let serial_port = Arc::clone(&serial_port);
        thread::spawn(move || loop {
            match serial_port.read_u8() /*read_u8 blocks*/ {
                Ok(byte) => tx.send(Notification::SerialInput(byte)).unwrap(),
                _ => panic!(),
            }
        });
    }

    // Dualshock PS4 controller events
    {
        let tx = tx.clone();
        thread::spawn(move || loop {
            // TODO: is it blocking? If not, it does not need a separate thread
            match device.get_event() {
                Err(error) => match error {
                    joydev::Error::QueueEmpty => (), // TODO: wait?
                    _ => panic!(
                        "{}: {:?}",
                        "called `Result::unwrap()` on an `Err` value", &error
                    ),
                },
                Ok(event) => match event {
                    DeviceEvent::Axis(event) => {
                        tx.send(Notification::ControllerAxis(event)).unwrap()
                    }
                    DeviceEvent::Button(event) => {
                        tx.send(Notification::ControllerButton(event)).unwrap()
                    }
                },
            }
        });
    }
    // consumer loop
    {
        /*recv() blocks*/
        while let Ok(notification) = rx.recv() {
            match notification {
                Notification::SerialInput(_byte) => (),
                Notification::ControllerButton(button_event) => {
                    match button_event.button() {
                        // see: https://gitlab.com/gm666q/joydev-rs/-/blob/master/joydev/src/event_codes/key.rs
                        Key::ButtonNorth => {
                            serial_port.write_u8(b'f')?;
                        }
                        Key::ButtonSouth => {
                            serial_port.write_u8(b's')?;
                        }
                        _ => (), // ignore rest
                    }
                }
                Notification::ControllerAxis(axis_event) => match axis_event.axis() {
                    AbsoluteAxis::LeftX => {
                        let _value = axis_event.value();
                    }
                    _ => (), // ignore rest
                },
            }
        }
    }
    unreachable!()
}

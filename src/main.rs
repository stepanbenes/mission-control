extern crate joydev;
mod serial_port;
use joydev::{event_codes::AbsoluteAxis, event_codes::Key, Device, DeviceEvent, GenericEvent};
use serial_port::*;
use std::sync::{mpsc::channel, Arc};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Notification {
    ControllerButton(joydev::ButtonEvent),
    ControllerAxis(joydev::AxisEvent),
    SerialInput(u8),
    //NetworkCommand(String),
}

// how to run: 1. connect dualshock4 to raspberry
//             2. sudo ds4drv --hidraw &
//             3. sudo ./dualshock

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // open serial port
    let serial_port = Arc::new(SerialPort::open("/dev/ttyACM0")?);
    let serial_port_2 = Arc::new(SerialPort::open("/dev/ttyACM0")?);

    serial_port.write_u8(7u8)?;
    let _ = serial_port_2.read_u8()?;

    // open joistick controller
    let device = Device::open("/dev/input/js0")?;

    // create communication channel
    let (tx, rx) = channel::<Notification>();

    // read serial port
    {
        let tx = tx.clone();
        let serial_port = Arc::clone(&serial_port);
        thread::spawn(move || loop {
            match serial_port.read_u8() {
                Ok(byte) => {
                    println!("Received char: {}", byte as char);
                    tx.send(Notification::SerialInput(byte)).unwrap();
                }
                Err(_) => () // continue
                //_ => panic!("serial_port.read_u8() failed"),
            }
            thread::sleep(Duration::from_millis(10));
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
                        println!("Axis event: {:?}", event);
                        tx.send(Notification::ControllerAxis(event)).unwrap()
                    }
                    DeviceEvent::Button(event) => {
                        println!("Button event: {:?}", event);
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
            println!("notification: {:?}", notification);
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

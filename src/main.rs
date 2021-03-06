extern crate joydev;
mod non_blocking_serial_port;
use joydev::{event_codes::AbsoluteAxis, event_codes::Key, Device, DeviceEvent, GenericEvent};
use non_blocking_serial_port::*;
use std::sync::{mpsc::channel, Arc};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Notification {
    ControllerButton(joydev::ButtonEvent),
    ControllerAxis(joydev::AxisEvent),
    SerialInput(u8),
    //NetworkCommand(String), // TODO: add network communication
}

// how to run: 1. connect dualshock4 to raspberry
//             2. sudo ds4drv --hidraw &
//             3. sudo ./mission-control

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // open serial port
    let serial_port = Arc::new(NonBlockingSerialPort::open("/dev/ttyACM0")?);

    // open joistick controller
    let device = Device::open("/dev/input/js0")?;

    // create communication channel
    let (tx, rx) = channel::<Notification>(); // TODO: is channel necessary if threads are not necessary?

    // listen to serial port events
    {
        let tx = tx.clone();
        let serial_port = Arc::clone(&serial_port);
        thread::spawn(move || loop {
            match serial_port.try_read_u8() {
                Ok(Some(byte)) => {
                    println!("Received char: {}", byte as char);
                    tx.send(Notification::SerialInput(byte)).unwrap();
                }
                Ok(None) => (),
                Err(_) => panic!("serial_port.try_read_u8() failed"),
            }

            // wait for some time to not consume 100% thread time
            thread::sleep(Duration::from_millis(20)); // longer delay?
        });
    }

    // listen to serial port events and dualshock PS4 controller events
    {
        let tx = tx.clone();
        thread::spawn(move || loop {
            match device.get_event() {
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
                Err(error) => match error {
                    joydev::Error::QueueEmpty => (),
                    _ => panic!(
                        "{}: {:?}",
                        "called `Result::unwrap()` on an `Err` value", &error
                    ),
                },
            }
        });
    }

    // consumer loop
    {
        /*recv() blocks*/
        loop {
            if let Ok(notification) = rx.try_recv() {
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
                            _ => (),
                        }
                    }
                    Notification::ControllerAxis(axis_event) => match axis_event.axis() {
                        AbsoluteAxis::LeftX => {
                            let _value = axis_event.value();
                        }
                        _ => (),
                    },
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    unreachable!()
}

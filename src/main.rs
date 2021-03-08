mod non_blocking_serial_port;

use non_blocking_serial_port::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::channel,
    Arc,
};
use std::thread;
use std::time::Duration;

use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

use gilrs::{Button, Event, Gilrs};

#[derive(Debug)]
enum Notification {
    // ControllerButton(joydev::ButtonEvent),
    // ControllerAxis(joydev::AxisEvent),
    SerialInput(u8),
    //NetworkCommand(String), // TODO: add network communication
    TerminationSignal(i32),
}

// how to run: 1. connect dualshock4 to raspberry
//             2. sudo ds4drv --hidraw &
//             3. sudo ./mission-control

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // open serial port
    let serial_port = Arc::new(NonBlockingSerialPort::open("/dev/ttyACM0")?);

    // create communication channel
    let (tx, rx) = channel::<Notification>(); // TODO: is channel necessary if threads are not necessary?

    // open joystick controller
    let mut gilrs = Gilrs::new()?;

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    // setup interrupt signals
    let is_running = Arc::new(AtomicBool::new(true));
    {
        let r = is_running.clone();
        let tx = tx.clone();
        let mut signals = Signals::new(TERM_SIGNALS)?;
        thread::spawn(move || {
            for signal in signals.forever() {
                r.store(false, Ordering::SeqCst); // tell other threads to shut down
                tx.send(Notification::TerminationSignal(signal)).unwrap();
                break; // stop this thread
            }
        });
    }

    // listen to serial port events
    let serial_port_thread;
    {
        let tx = tx.clone();
        let serial_port = Arc::clone(&serial_port);
        let is_running = is_running.clone();
        serial_port_thread = thread::spawn(move || {
            while is_running.load(Ordering::SeqCst) {
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
            }
        });
    }

    // listen to dualshock PS4 controller events
    // let joystick_thread;
    // {
    //     let tx = tx.clone();
    //     let is_running = is_running.clone();
    //     joystick_thread = thread::spawn(move || {
    //         while is_running.load(Ordering::SeqCst) {
    //             match joystick.get_event() {
    //                 // TODO: this is problem, it is non-blocking and the loop is consuming 100% CPU time
    //                 Ok(event) => match event {
    //                     DeviceEvent::Axis(event) => {
    //                         println!("Axis event: {:?}", event);
    //                         tx.send(Notification::ControllerAxis(event)).unwrap()
    //                     }
    //                     DeviceEvent::Button(event) => {
    //                         println!("Button event: {:?}", event);
    //                         tx.send(Notification::ControllerButton(event)).unwrap()
    //                     }
    //                 },
    //                 Err(error) => match error {
    //                     joydev::Error::QueueEmpty => (),
    //                     _ => panic!(
    //                         "{}: {:?}",
    //                         "called `Result::unwrap()` on an `Err` value", &error
    //                     ),
    //                 },
    //             }
    //         }
    //     });
    // }

    // notification processing loop
    {
        'consumer_loop: loop {
            /*recv() blocks*/
            if let Ok(notification) = rx.recv() {
                println!("notification: {:?}", notification);
                match notification {
                    Notification::SerialInput(_byte) => {}
                    // Notification::ControllerButton(button_event) => {
                    //     match button_event.button() {
                    //         see: https://gitlab.com/gm666q/joydev-rs/-/blob/master/joydev/src/event_codes/key.rs
                    //         Key::ButtonNorth => {
                    //             serial_port.write_u8(b'f')?;
                    //         }
                    //         Key::ButtonSouth => {
                    //             serial_port.write_u8(b's')?;
                    //         }
                    //         _ => (),
                    //     }
                    // }
                    // Notification::ControllerAxis(axis_event) => match axis_event.axis() {
                    //     AbsoluteAxis::LeftX => {
                    //         let _value = axis_event.value();
                    //     }
                    //     _ => (),
                    // },
                    Notification::TerminationSignal(signal) => {
                        eprintln!("Received signal {:?}", signal);
                        break 'consumer_loop;
                    }
                }
            }
        }
    }

    serial_port_thread
        .join()
        .expect("The serial port thread being joined has panicked.");
    // joystick_thread
    //     .join()
    //     .expect("The joystick thread being joined has panicked.");

    println!("all threads exited.");

    Ok(())
}

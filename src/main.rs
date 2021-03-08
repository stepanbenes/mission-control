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

use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
use gilrs::{Button, Event, EventType, Gamepad, Gilrs};

use string_error::static_err;

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

    // interrupt watcher loop
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

    // producer loop
    let producer_thread;
    {
        let tx = tx.clone();
        let serial_port = Arc::clone(&serial_port);
        let is_running = is_running.clone();
        producer_thread = thread::spawn(move || {
            // open joystick controller
            let mut gilrs = Gilrs::new().unwrap(); // TODO: catch errors

            // get first connected gamepad
            let (_, gamepad) = gilrs
                .gamepads()
                .next()
                .ok_or(static_err("No gamepad is connected."))
                .unwrap(); // TODO: catch errors
            println!(
                "{} is {:?}; ff: {}",
                gamepad.name(),
                gamepad.power_info(),
                gamepad.is_ff_supported()
            );

            while is_running.load(Ordering::SeqCst) {
                // read serial port
                match serial_port.try_read_u8() {
                    Ok(Some(byte)) => {
                        println!("Received char: {}", byte as char);
                        tx.send(Notification::SerialInput(byte)).unwrap();
                    }
                    Ok(None) => (),
                    Err(_) => panic!("serial_port.try_read_u8() failed"),
                }

                // read gamepad
                while let Some(Event {
                    id: gamepad_id,
                    event,
                    time: _,
                }) = gilrs.next_event()
                {
                    println!("{:?}", event);
                    match event {
                        EventType::ButtonChanged(Button::South, _value, _nec) => {
                            let duration = Ticks::from_ms(150);
                            let effect = EffectBuilder::new()
                                .add_effect(BaseEffect {
                                    kind: BaseEffectType::Strong { magnitude: 60_000 },
                                    scheduling: Replay {
                                        play_for: duration,
                                        with_delay: duration * 3,
                                        ..Default::default()
                                    },
                                    envelope: Default::default(),
                                })
                                .add_effect(BaseEffect {
                                    kind: BaseEffectType::Weak { magnitude: 60_000 },
                                    scheduling: Replay {
                                        after: duration * 2,
                                        play_for: duration,
                                        with_delay: duration * 3,
                                    },
                                    ..Default::default()
                                })
                                .gamepads(&[gamepad_id])
                                .finish(&mut gilrs)
                                .unwrap();
                            effect.play().unwrap();
                            thread::sleep(Duration::from_secs(11));
                        }
                        _ => {}
                    }
                }

                // wait for some time to not consume 100% thread time
                thread::sleep(Duration::from_millis(50)); // longer delay?
            }
        });
    }

    // consumer loop
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

    producer_thread
        .join()
        .expect("The producer thread being joined has panicked.");

    println!("all threads exited.");

    Ok(())
}

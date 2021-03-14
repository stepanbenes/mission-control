mod non_blocking_serial_port;

use non_blocking_serial_port::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver, Sender},
    Arc,
};
use std::thread;
use std::time::Duration;

use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
use gilrs::{Button, Event, EventType::*, Gilrs};

//use string_error::static_err;

#[derive(Debug)]
enum Notification {
    GamepadButton(Button),
    //GamepadAxis(joydev::AxisEvent),
    SerialInput(u8),
    //NetworkMessage(String), // TODO: add network communication (use tungstenite)
    //TerminationSignal(i32),
    //ImageTaken { uri: String },
    //DistanceMeasured
    //ArrivedToPosition
}

// #[derive(Debug)]
// enum Command {
//     TakeImage,
//     MeasureDistance,
//     GoToPosition
// }

// how to run: 1. connect dualshock4 to raspberry
//             2. sudo ds4drv --hidraw &
//             3. sudo ./mission-control

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // open serial port
    let serial_port = NonBlockingSerialPort::open("/dev/ttyACM0")?;

    // create communication channel
    let (tx, rx) = channel::<Notification>();

    // interrupt watcher loop
    let is_running = Arc::new(AtomicBool::new(true));
    {
        let r = is_running.clone();
        let mut signals = Signals::new(TERM_SIGNALS)?;
        thread::spawn(move || {
            for signal in signals.forever() {
                r.store(false, Ordering::SeqCst); // tell other threads to shut down
                eprintln!("Received signal {:?}", signal);
                break; // stop this thread
            }
        });
    }

    // consumer loop
    {
        let mut gilrs = Gilrs::new().expect("Gilrs could not be created");
        while is_running.load(Ordering::SeqCst) {
            produce_serial_port_notifications(&serial_port, &tx);
            produce_gamepad_notifications(&mut gilrs, &tx);
            consume_all_notifications(&rx, &serial_port, &mut gilrs);
            println!("tick");
            thread::sleep(Duration::from_millis(20)); // longer delay?
        }
    }

    // TODO: join all threads
    // producer_thread
    //     .join()
    //     .expect("The producer thread being joined has panicked.");

    println!("all threads exited.");

    Ok(())
}

fn produce_serial_port_notifications(
    serial_port: &NonBlockingSerialPort,
    sender: &Sender<Notification>,
) {
    match serial_port.try_read_u8() {
        Ok(Some(byte)) => {
            println!("Received char: {}", byte as char);
            sender
                .send(Notification::SerialInput(byte))
                .expect("tx.send failed.");
        }
        Ok(None) => (),
        Err(_) => panic!("serial_port.try_read_u8() failed"),
    }
}

fn produce_gamepad_notifications(gilrs: &mut Gilrs, sender: &Sender<Notification>) {
    while let Some(Event {
        id: gamepad_id,
        event,
        time: _,
    }) = gilrs.next_event()
    {
        println!("{:?}", event);
        match event {
            Connected => {
                let gamepad = gilrs
                    .connected_gamepad(gamepad_id)
                    .expect("gamepad should be connected but it is not.");
                println!(
                    "{} is connected; power info: {:?}; force feedback: {};",
                    gamepad.name(),
                    gamepad.power_info(),
                    if gamepad.is_ff_supported() {
                        "supported"
                    } else {
                        "not supported"
                    }
                );
            }
            Disconnected => {
                let disconnected_gamepad = gilrs.gamepad(gamepad_id);
                println!("{} disconnected;", disconnected_gamepad.name());
            }
            ButtonPressed(button, _) => {
                sender.send(Notification::GamepadButton(button)).unwrap();
            }
            ButtonRepeated(_, _) => {}
            ButtonReleased(_, _) => {}
            AxisChanged(_axis, _value, _code) => {}
            ButtonChanged(Button::South, _value, _code) => {
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
                    .finish(gilrs)
                    .unwrap();
                effect.play().unwrap();
                thread::sleep(Duration::from_secs(11)); // must wait to finishe effect before reading next event
            }
            ButtonChanged(_, _, _) => {}
            Dropped => { /*ignore*/ }
        }
    }
}

fn consume_all_notifications(
    receiver: &Receiver<Notification>,
    serial_port: &NonBlockingSerialPort,
    _gilrs: &mut Gilrs, // TODO: use for force feedback
) {
    /*recv() blocks*/
    while let Ok(notification) = receiver.try_recv() {
        println!("notification: {:?}", notification);
        match notification {
            Notification::SerialInput(_byte) => {}
            Notification::GamepadButton(button) => {
                match button {
                    //see: https://gitlab.com/gm666q/joydev-rs/-/blob/master/joydev/src/event_codes/key.rs
                    Button::North => {
                        serial_port.write_u8(b'f').unwrap();
                    }
                    Button::South => {
                        serial_port.write_u8(b's').unwrap();
                    }
                    _ => {}
                }
            }
        }
    }
}

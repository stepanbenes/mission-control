extern crate serial;

use serial::prelude::SerialPort;
use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

pub struct SerialPortX {
    port: Mutex<serial::SystemPort>,
}

// Default settings of Arduino
// see: https://www.arduino.cc/en/Serial/Begin
const PORT_SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud9600,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

impl SerialPortX {
    pub fn open(port_name: &str) -> Result<Self, Error> {
        let mut port = serial::open(port_name)?;
        port.configure(&PORT_SETTINGS).unwrap();
        // timeout of 30s
        port.set_timeout(Duration::from_secs(30)).unwrap();
        Ok(SerialPortX {
            port: Mutex::new(port),
        })
    }

    pub fn read_u8(&self) -> Result<u8, Error> {
        let mut read_buffer = [0u8; 1];
        self.port.lock().unwrap().read_exact(&mut read_buffer)?;
        Ok(read_buffer[0] as u8)
    }

    pub fn write_u8(&self, data: u8) -> Result<usize, Error> {
        let buffer = [data];
        let num_bytes = self.port.lock().unwrap().write(&buffer)?;
        Ok(num_bytes)
    }
}

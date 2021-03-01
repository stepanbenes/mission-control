extern crate serial;

use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

pub struct SerialPort {
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

impl SerialPort {
    pub fn open(port_name: &str) -> Result<Self, Error> {
        let mut port = serial::open(port_name)?;
        serial::SerialPort::configure(&mut port, &PORT_SETTINGS)?;
        // timeout of 30s
        serial::SerialPort::set_timeout(&mut port, Duration::from_secs(30))?;

        Ok(SerialPort {
            port: Mutex::new(port),
        })
    }

    pub fn read_u8(&self) -> Result<u8, Error> {
        let mut read_buffer = [0u8];
        self.port.lock().unwrap().read_exact(&mut read_buffer)?;
        if read_buffer.len() == 0 {
            Err(Error::from(std::io::ErrorKind::WouldBlock))
        } else {
            Ok(read_buffer[0] as u8)
        }
    }

    pub fn write_u8(&self, data: u8) -> Result<usize, Error> {
        let buffer = [data];
        let num_bytes = self.port.lock().unwrap().write(&buffer)?;
        Ok(num_bytes)
    }
}

extern crate serial;

use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::sync::Mutex;

pub struct SerialPort {
    port: Mutex<serial::SystemPort>,
}

impl SerialPort {
    pub fn open(port_name: &str) -> Result<Self, Error> {
        Ok(SerialPort {
            port: Mutex::new(serial::open(port_name)?),
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

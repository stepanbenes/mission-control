extern crate mio_serial;

use mio_serial::SerialPort;
use std::io::Read;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::sync::Mutex;
use std::time::Duration;

pub struct SerialPortX {
    port: Mutex<mio_serial::Serial>,
}

// Default settings of Arduino
// see: https://www.arduino.cc/en/Serial/Begin
const PORT_SETTINGS: mio_serial::SerialPortSettings = mio_serial::SerialPortSettings {
    baud_rate: 9600,
    data_bits: mio_serial::DataBits::Eight,
    parity: mio_serial::Parity::None,
    stop_bits: mio_serial::StopBits::One,
    flow_control: mio_serial::FlowControl::None,
    timeout: Duration::from_secs(30),
};

impl SerialPortX {
    pub fn open(port_name: &str) -> Result<Self, Error> {
        // https://docs.rs/mio-serial/3.3.1/mio_serial/trait.SerialPort.html#tymethod.bytes_to_read

        let mio_serial_port = mio_serial::Serial::from_path(port_name, &PORT_SETTINGS)?;

        Ok(SerialPortX {
            port: Mutex::new(mio_serial_port),
        })
    }

    pub fn read_u8(&self) -> Result<u8, Error> {
        let byte_count = self.port.lock().unwrap().bytes_to_read()?;
        if byte_count > 0 {
            let mut read_buffer = [0u8];
            self.port.lock().unwrap().read_exact(&mut read_buffer)?;
            Ok(read_buffer[0] as u8)
        } else {
            Err(Error::from(ErrorKind::WouldBlock))
        }
    }

    pub fn write_u8(&self, data: u8) -> Result<usize, Error> {
        let buffer = [data];
        let num_bytes = self.port.lock().unwrap().write(&buffer)?;
        Ok(num_bytes)
    }
}

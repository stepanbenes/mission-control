extern crate mio_serial;

use mio_serial::SerialPort;
use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

pub struct NonBlockingSerialPort {
    port: Mutex<mio_serial::Serial>,
}

impl NonBlockingSerialPort {
    pub fn open(port_name: &str) -> Result<Self, Error> {
        // https://docs.rs/mio-serial/3.3.1/mio_serial/trait.SerialPort.html#tymethod.bytes_to_read
        const PORT_SETTINGS: mio_serial::SerialPortSettings = mio_serial::SerialPortSettings {
            baud_rate: 9600,
            data_bits: mio_serial::DataBits::Eight,
            parity: mio_serial::Parity::None,
            stop_bits: mio_serial::StopBits::One,
            flow_control: mio_serial::FlowControl::None,
            timeout: Duration::from_secs(30),
        };
        let mio_serial_port = mio_serial::Serial::from_path(port_name, &PORT_SETTINGS)?;

        Ok(NonBlockingSerialPort {
            port: Mutex::new(mio_serial_port),
        })
    }

    pub fn try_read_u8(&self) -> Result<Option<u8>, Error> {
        let byte_count = self.port.lock().unwrap().bytes_to_read()?;
        if byte_count > 0 {
            let mut read_buffer = [0u8];
            self.port.lock().unwrap().read_exact(&mut read_buffer)?;
            Ok(Some(read_buffer[0] as u8))
        } else {
            Ok(None)
        }
    }

    pub fn write_u8(&self, data: u8) -> Result<usize, Error> {
        let buffer = [data];
        let num_bytes = self.port.lock().unwrap().write(&buffer)?;
        Ok(num_bytes)
    }
}

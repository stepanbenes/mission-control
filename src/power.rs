use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub struct PowerStatus {
    capacity: u32,
    status: String,
    device_type: String,
}

impl Display for PowerStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}|{}|{}]", self.device_type, self.capacity, self.status)
    }
}

pub fn check_power_status(device_path: &str) -> Result<Option<PowerStatus>, Box<dyn std::error::Error>> {
    let map = build_device_address_map()?;
    if let Some(address) = map.get(device_path) {
        if let Some(directory) = find_device_directory(Path::new("/sys/class/power_supply"), address)? {
            // capacity
            let filepath = Path::new(&directory).join("capacity");
            let file_content = std::fs::read_to_string(&filepath)?;
            let capacity = file_content.trim().parse::<u32>()?;
            // status
            let filepath = Path::new(&directory).join("status");
            let file_content = std::fs::read_to_string(&filepath)?;
            let status = file_content.trim().to_string();
            // type
            let filepath = Path::new(&directory).join("type");
            let file_content = std::fs::read_to_string(&filepath)?;
            let device_type = file_content.trim().to_string();
            return Ok(
                Some(PowerStatus {
                    capacity,
                    status,
                    device_type,
                }));
        }
    }
	Ok(None)
}

fn find_device_directory(dir: &Path, device_address: &str) -> io::Result<Option<String>> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let path_string = path.into_os_string().into_string().unwrap();
                if path_string.ends_with(device_address) {
                    return Ok(Some(path_string));
                }
            }
        }
    }
    Ok(None)
}

fn build_device_address_map() -> io::Result<HashMap::<String, String>> {
    let mut device_address_map = HashMap::<String, String>::new();
	let lines = read_lines("/proc/bus/input/devices")?;
    let mut current_address: Option<String> = None;
    for line_result in lines {
        let line = line_result?;
        if let Some(captures) = crate::DEVICE_INFO_ADDRESS_LINE_REGEX.captures_iter(&line).next() { // Uniq=address
            current_address = Some(captures[1].to_owned());
        }
        else if let Some(captures) = crate::DEVICE_INFO_HANDLERS_LINE_REGEX.captures_iter(&line).next() { // Handlers=...
            let handlers = &captures[1];
            for handler_capture in crate::EVENT_FILE_REGEX.captures_iter(handlers) {
                let device_name = &handler_capture[0];
                if let Some(ref current_address) = current_address {
                    let device_path = format!("/dev/input/{}", device_name);
                    device_address_map.insert(device_path, current_address.to_owned());
                }
            }
        }
    }
    Ok(device_address_map)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
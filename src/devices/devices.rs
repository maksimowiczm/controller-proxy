use regex::Regex;
use std::fs::File;
use std::io::Read;

pub struct Devices(Vec<Device>);

pub struct Device {
    name: String,
    handlers: Vec<String>,
}

impl Device {
    pub fn from_str(description: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let re_name = Regex::new(r#"N: Name="(?<name>.+)""#)?;
        let name = re_name.captures(&description).ok_or("noname")?["name"].to_owned();

        let re_handlers = Regex::new(r#"H: Handlers=(?<handlers>.+)"#)?;
        let handlers = if let Some(captures) = re_handlers.captures(&description) {
            captures["handlers"]
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        Ok(Device { name, handlers })
    }

    pub fn get_event_handler(&self) -> Result<File, Box<dyn std::error::Error>> {
        let re_event = Regex::new(r"(?<event>event\d+)")?;
        let event_handler = self
            .handlers
            .iter()
            .find(|handler| re_event.captures(handler).is_some())
            .ok_or("Device has no event handler")?;

        let path = format!("/dev/input/{event_handler}");
        let dev_file = File::open(&path).or(Err(format!("Can not read from {path}")))?;

        return Ok(dev_file);
    }

    pub fn from_proc_file(re: Regex) -> Result<Self, Box<dyn std::error::Error>> {
        let mut input_devices_file = File::open("/proc/bus/input/devices")?;
        let mut input_devices = String::new();
        input_devices_file.read_to_string(&mut input_devices)?;
        drop(input_devices_file); // do not waste resources

        let device = input_devices
            .split("\n\n")
            .flat_map(|str| Device::from_str(str))
            .find(|device| re.captures(&*device.name).is_some())
            .ok_or("Device not found")?;

        Ok(device)
    }
}

impl Devices {
    pub fn from_proc_file() -> Result<Self, Box<dyn std::error::Error>> {
        let mut input_devices_file = File::open("/proc/bus/input/devices")?;
        let mut input_devices = String::new();
        input_devices_file.read_to_string(&mut input_devices)?;
        drop(input_devices_file); // do not waste resources

        let devices = input_devices
            .split("\n\n")
            .flat_map(|str| Device::from_str(str))
            .collect::<Vec<_>>();

        Ok(Devices(devices))
    }

    pub fn get_device(&self, re: &Regex) -> Option<&Device> {
        self.0.iter().find(|device| re.is_match(&*device.name))
    }
}

use std::error::Error;
use std::path::Path;

use serde_derive::Deserialize;

use crate::utils::load_file;

#[derive(Debug, Deserialize)]
pub struct Mcu {
    #[serde(rename = "IP", default)]
    ip: Vec<IP>,
    #[serde(rename = "E2prom")]
    eeprom_size_bytes: String,
}

impl Mcu {
    pub fn load<P: AsRef<Path>>(db_dir: P, mcu_name: &str) -> Result<Self, Box<dyn Error>> {
        load_file(db_dir, format!("{}.xml", mcu_name))
    }

    pub fn get_ip(&self, name: &str) -> Option<&IP> {
        self.ip.iter().find(|v| v.name == name)
    }

    /// Return the EEPROM size in bytes
    pub fn get_eeprom_size(&self) -> Option<u32> {
        self.eeprom_size_bytes.parse().ok()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IP {
    instance_name: String,
    name: String,
    version: String,
}

impl IP {
    pub fn get_version(&self) -> &str {
        &self.version
    }
}

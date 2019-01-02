use serde;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Mcu {
    #[serde(rename = "IP", default)]
    ip: Vec<IP>,
}

impl Mcu {
    pub fn get_ip(&self, name: &str) -> Option<&IP> {
        self.ip.iter().find(|v| v.name == name)
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

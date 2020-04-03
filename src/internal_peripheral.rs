use std::error::Error;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::Deserialize;

use crate::utils::load_file;

#[derive(Debug, Deserialize)]
pub(crate) struct PossibleValue {
    #[serde(rename = "$value")]
    pub(crate) val: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SpecificParameter {
    name: String,
    possible_value: PossibleValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PinSignal {
    name: String,
    specific_parameter: SpecificParameter,
}

impl PinSignal {
    fn get_af_value(&self) -> &str {
        self.specific_parameter
            .possible_value
            .val
            .split('_')
            .collect::<Vec<_>>()[1]
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "GPIO_Pin", rename_all = "PascalCase")]
pub struct GPIOPin {
    port_name: String,
    name: String,
    specific_parameter: Vec<SpecificParameter>,
    pin_signal: Option<Vec<PinSignal>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "IP")]
pub struct IpGPIO {
    #[serde(rename = "GPIO_Pin")]
    pub(crate) gpio_pin: Vec<GPIOPin>,
}

impl IpGPIO {
    pub fn load<P: AsRef<Path>>(db_dir: P, version: &str) -> Result<Self, Box<dyn Error>> {
        load_file(db_dir, format!("IP/GPIO-{}_Modes.xml", version))
    }
}

lazy_static! {
    static ref USART_RX: Regex = Regex::new("(LP)?US?ART._RX").unwrap();
    static ref USART_TX: Regex = Regex::new("(LP)?US?ART._TX").unwrap();
    static ref SPI_MOSI: Regex = Regex::new("SPI._MOSI").unwrap();
    static ref SPI_MISO: Regex = Regex::new("SPI._MISO").unwrap();
    static ref SPI_SCK: Regex = Regex::new("SPI._SCK").unwrap();
    static ref I2C_SCL: Regex = Regex::new("I2C._SCL").unwrap();
    static ref I2C_SDA: Regex = Regex::new("I2C._SDA").unwrap();
}

impl GPIOPin {
    pub fn get_name(&self) -> Option<String> {
        let gpio_pin = self
            .specific_parameter
            .iter()
            .find(|v| v.name == "GPIO_Pin");
        match gpio_pin {
            Some(v) => {
                let num = v.possible_value.val.split('_').collect::<Vec<_>>()[2];
                Some(format!("{}{}", &self.port_name, num))
            }
            None => None,
        }
    }

    pub fn get_af_modes(&self) -> Vec<String> {
        let mut res = Vec::new();
        if let Some(ref v) = self.pin_signal {
            for sig in v {
                let per = sig.name.split('_').collect::<Vec<_>>()[0];
                if USART_RX.is_match(&sig.name) {
                    res.push(format!("{}: RxPin<{}>", sig.get_af_value(), per));
                }
                if USART_TX.is_match(&sig.name) {
                    res.push(format!("{}: TxPin<{}>", sig.get_af_value(), per));
                }
                if SPI_MOSI.is_match(&sig.name) {
                    res.push(format!("{}: MosiPin<{}>", sig.get_af_value(), per));
                }
                if SPI_MISO.is_match(&sig.name) {
                    res.push(format!("{}: MisoPin<{}>", sig.get_af_value(), per));
                }
                if SPI_SCK.is_match(&sig.name) {
                    res.push(format!("{}: SckPin<{}>", sig.get_af_value(), per));
                }
                if I2C_SCL.is_match(&sig.name) {
                    res.push(format!("{}: SclPin<{}>", sig.get_af_value(), per));
                }
                if I2C_SDA.is_match(&sig.name) {
                    res.push(format!("{}: SdaPin<{}>", sig.get_af_value(), per));
                }
            }
        }
        res
    }
}

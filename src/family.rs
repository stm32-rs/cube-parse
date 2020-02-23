use std::error::Error;
use std::path::Path;

use serde_derive::Deserialize;

use crate::utils::load_file;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Families {
    #[serde(rename = "Family")]
    families: Vec<Family>,
}

/// A MCU family (e.g. "STM32F0" or "STM32L3").
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Family {
    pub name: String,
    #[serde(rename = "SubFamily")]
    sub_families: Vec<SubFamily>,
}

/// A MCU subfamily (e.g. "STM32F0x0 Value Line").
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SubFamily {
    pub name: String,
    #[serde(rename = "Mcu")]
    pub mcus: Vec<Mcu>,
}

/// A MCU (e.g. STM32L071KBTx).
///
/// Note that multiple MCUs (with unique `ref_name`) share a common name. For
/// example:
///
/// - `<Mcu Name="STM32L071K(B-Z)Tx" PackageName="LQFP32" RefName="STM32L071KBTx" RPN="STM32L071KB">`
/// - `<Mcu Name="STM32L071K(B-Z)Tx" PackageName="LQFP32" RefName="STM32L071KZTx" RPN="STM32L071KZ">`
///
/// Both MCUs share the same name, but the ref name is different.
///
/// The meaning of the name, using the `STM32L071KBTx` as an example:
///
/// |Part |Meaning |
/// |-----|--------|
/// |STM32|Family  |
/// | L   |Type    |
/// | 0   |Core    |
/// | 71  |Line    |
/// | K   |Pincount|
/// | B   |Flash   |
/// | T   |Package |
/// | x   |Temp    |
/// |-----|--------|
///
/// See https://ziutek.github.io/2018/05/07/stm32_naming_scheme.html for more details.
///
/// Note that sometimes there are exceptions from this naming rule.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Mcu {
    pub name: String,
    pub package_name: String,
    pub ref_name: String,
}

impl Families {
    pub fn load<P: AsRef<Path>>(db_dir: P) -> Result<Self, Box<dyn Error>> {
        load_file(db_dir, "families.xml")
    }
}

impl<'a> IntoIterator for &'a Families {
    type Item = &'a Family;
    type IntoIter = std::slice::Iter<'a, Family>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.families.iter()
    }
}

impl<'a> IntoIterator for &'a Family {
    type Item = &'a SubFamily;
    type IntoIter = std::slice::Iter<'a, SubFamily>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.sub_families.iter()
    }
}

impl<'a> IntoIterator for &'a SubFamily {
    type Item = &'a Mcu;
    type IntoIter = std::slice::Iter<'a, Mcu>;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.mcus.iter()
    }
}

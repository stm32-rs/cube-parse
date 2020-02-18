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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Family {
    pub name: String,
    #[serde(rename = "SubFamily")]
    sub_families: Vec<SubFamily>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SubFamily {
    pub name: String,
    #[serde(rename = "Mcu")]
    mcus: Vec<Mcu>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Mcu {
    pub name: String,
    package_name: String,
    ref_name: String,
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

use std::{collections::HashMap, env, path::Path};

use alphanumeric_sort::compare_str;
use clap::{App, Arg};
use lazy_static::lazy_static;
use regex::Regex;

mod family;
mod internal_peripheral;
mod mcu;
mod utils;

#[derive(Debug, PartialEq)]
enum GenerateTarget {
    Features,
    PinMappings,
    EepromSizes,
}

lazy_static! {
    // Note: Version >1.0 is not currently supported
    static ref GPIO_VERSION: Regex = Regex::new("^([^_]*)_gpio_v1_0$").unwrap();
}

/// Convert a GPIO IP version (e.g. "STM32L152x8_gpio_v1_0") to a feature name
/// (e.g. "io-STM32L152x8").
fn gpio_version_to_feature(version: &str) -> Result<String, String> {
    if let Some(captures) = GPIO_VERSION.captures(version) {
        Ok(format!("io-{}", captures.get(1).unwrap().as_str()))
    } else {
        Err(format!("Could not parse version {:?}", version))
    }
}

/// Get the EEPROM size feature for a certain size.
fn eeprom_size_to_feature(size: u32) -> String {
    format!("eeprom-{}", size)
}

fn main() -> Result<(), String> {
    let args = App::new("cube-parse")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Extract AF modes on MCU pins from the database files provided with STM32CubeMX")
        .author(&*env!("CARGO_PKG_AUTHORS").replace(":", ", "))
        .arg(
            Arg::with_name("db_dir")
                .short("d")
                .help("Path to the CubeMX MCU database directory")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("generate")
                .help("What to generate")
                .takes_value(true)
                .possible_values(&["features", "pin_mappings", "eeprom_sizes"])
                .required(true),
        )
        .arg(
            Arg::with_name("mcu_family")
                .help("The MCU family to extract, e.g. \"STM32L0\"")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    // Process args
    let db_dir = Path::new(args.value_of("db_dir").unwrap());
    let mcu_family = args.value_of("mcu_family").unwrap();
    let generate = match args.value_of("generate").unwrap() {
        "features" => GenerateTarget::Features,
        "pin_mappings" => GenerateTarget::PinMappings,
        "eeprom_sizes" => GenerateTarget::EepromSizes,
        _ => unreachable!(),
    };

    // Load families
    let families = family::Families::load(&db_dir)
        .map_err(|e| format!("Could not load families XML: {}", e))?;

    // Find target family
    let family = (&families)
        .into_iter()
        .find(|v| v.name == mcu_family)
        .ok_or_else(|| format!("Could not find family {}", mcu_family))?;

    // MCU map
    //
    // This maps a MCU ref name to the corresponding `mcu::Mcu` instance.
    let mut mcu_map: HashMap<String, mcu::Mcu> = HashMap::new();

    // GPIO map
    //
    // The keys of this map are GPIO peripheral version strings (e.g.
    // "STM32L051_gpio_v1_0"), while the value is a Vec of MCU ref names.
    let mut mcu_gpio_map: HashMap<String, Vec<String>> = HashMap::new();

    // Package map
    //
    // The keys of this map are MCU ref names, the values are package names.
    let mut mcu_package_map: HashMap<String, String> = HashMap::new();

    // EEPROM size map
    //
    // The keys of this map are EEPROM sizes, the values are Vecs of MCU ref names.
    let mut mcu_eeprom_size_map: HashMap<u32, Vec<String>> = HashMap::new();

    // Iterate through subfamilies, then through MCUs. Fill the maps above with
    // aggregated data.
    for sf in family {
        for mcu in sf {
            // Load MCU data from the XML files
            let mcu_dat = mcu::Mcu::load(&db_dir, &mcu.name)
                .map_err(|e| format!("Could not load MCU data: {}", e))?;

            // Fill GPIO map
            let gpio_version = mcu_dat.get_ip("GPIO").unwrap().get_version().to_string();
            mcu_gpio_map
                .entry(gpio_version)
                .or_insert(vec![])
                .push(mcu.ref_name.clone());

            // Fill package map
            if mcu_family == "STM32L0" {
                // The stm32l0xx-hal has package based features
                mcu_package_map.insert(mcu.ref_name.clone(), mcu.package_name.clone());
            }

            // Fill EEPROM size map
            if let Some(size) = mcu_dat.get_eeprom_size() {
                mcu_eeprom_size_map
                    .entry(size)
                    .or_insert(vec![])
                    .push(mcu.ref_name.clone());
            }

            mcu_map.insert(mcu.ref_name.clone(), mcu_dat);
        }
    }

    match generate {
        GenerateTarget::Features => generate_features(
            &mcu_map,
            &mcu_gpio_map,
            &mcu_package_map,
            &mcu_eeprom_size_map,
            &mcu_family,
        )?,
        GenerateTarget::PinMappings => generate_pin_mappings(&mcu_gpio_map, &db_dir)?,
        GenerateTarget::EepromSizes => generate_eeprom_sizes(&mcu_eeprom_size_map)?,
    };

    Ok(())
}

lazy_static! {
    static ref FEATURE_DEPENDENCIES: HashMap<&'static str, HashMap<&'static str, &'static str>> = {
        let mut m = HashMap::new();

        // STM32L0
        let mut l0 = HashMap::new();
        l0.insert("^STM32L0.1", "stm32l0x1");
        l0.insert("^STM32L0.2", "stm32l0x2");
        l0.insert("^STM32L0.3", "stm32l0x3");
        m.insert("STM32L0", l0);

        m
    };
}

/// Generate all Cargo features
///
/// Feature categories:
///
/// - IO features (`io-*`)
/// - EEPROM features (`eeprom-*`)
///
/// Finally, the MCU features are printed, they act purely as aliases for the
/// other features.
fn generate_features(
    mcu_map: &HashMap<String, mcu::Mcu>,
    mcu_gpio_map: &HashMap<String, Vec<String>>,
    mcu_package_map: &HashMap<String, String>,
    mcu_eeprom_size_map: &HashMap<u32, Vec<String>>,
    mcu_family: &str,
) -> Result<(), String> {
    // IO features
    let mut io_features = mcu_gpio_map
        .keys()
        .map(|gpio| gpio_version_to_feature(gpio))
        .collect::<Result<Vec<String>, String>>()?;
    io_features.sort();
    println!("# Features based on the GPIO peripheral version");
    println!("# This determines the pin function mapping of the MCU");
    for feature in io_features {
        println!("{} = []", feature);
    }
    println!();

    // EEPROM sizes
    let mut eeprom_sizes = mcu_eeprom_size_map.keys().collect::<Vec<_>>();
    eeprom_sizes.sort();
    println!("# Features based on EEPROM size (in bytes)");
    for size in eeprom_sizes {
        println!("{} = []", eeprom_size_to_feature(*size));
    }
    println!();

    // Physical packages
    if !mcu_package_map.is_empty() {
        println!("# Physical packages");
        let mut packages = mcu_package_map
            .values()
            .map(|v| v.to_lowercase())
            .collect::<Vec<_>>();
        packages.sort_by(|a, b| compare_str(a, b));
        packages.dedup();
        for pkg in packages {
            println!("{} = []", pkg);
        }
        println!();
    }

    // MCU features
    let mut mcu_aliases = vec![];
    for (gpio, mcu_list) in mcu_gpio_map {
        let gpio_version_feature = gpio_version_to_feature(gpio).unwrap();
        for mcu in mcu_list {
            let mut dependencies = vec![];

            // Static feature dependencies
            if let Some(family) = FEATURE_DEPENDENCIES.get(mcu_family) {
                for (pattern, feature) in family {
                    if Regex::new(pattern).unwrap().is_match(&mcu) {
                        dependencies.push(feature.to_string());
                        break;
                    }
                }
            }

            // Package based feature
            if let Some(package) = mcu_package_map.get(mcu) {
                dependencies.push(package.to_lowercase());
            }

            // GPIO version feature
            dependencies.push(gpio_version_feature.clone());

            // EEPROM size
            if let Some(size) = mcu_map.get(mcu).unwrap().get_eeprom_size() {
                dependencies.push(eeprom_size_to_feature(size));
            }

            mcu_aliases.push(format!(
                "mcu-{} = [{}]",
                mcu,
                &dependencies.iter().map(|val| format!("\"{}\"", val)).fold(
                    String::new(),
                    |mut acc, x| {
                        if !acc.is_empty() {
                            acc.push_str(", ");
                        }
                        acc.push_str(&x);
                        acc
                    }
                )
            ));
        }
    }
    mcu_aliases.sort();
    println!("# MCU aliases");
    println!("#");
    println!("# Note: These are just aliases, they should not be used to directly feature gate");
    println!(
        "# functionality in the HAL! However, user code should usually depend on a MCU alias."
    );
    for alias in mcu_aliases {
        println!("{}", alias);
    }

    Ok(())
}

/// Generate the pin mappings for the target MCU family.
fn generate_pin_mappings(
    mcu_gpio_map: &HashMap<String, Vec<String>>,
    db_dir: &Path,
) -> Result<(), String> {
    let mut gpio_versions = mcu_gpio_map.keys().collect::<Vec<_>>();
    gpio_versions.sort();
    for gpio in gpio_versions {
        let gpio_version_feature = gpio_version_to_feature(&gpio)?;
        println!("#[cfg(feature = \"{}\")]", gpio_version_feature);
        let gpio_data = internal_peripheral::IpGPIO::load(db_dir, &gpio)
            .map_err(|e| format!("Could not load IP GPIO file: {}", e))?;
        render_pin_modes(&gpio_data);
        println!("\n");
    }
    Ok(())
}

/// Generate code containing the EEPROM size.
fn generate_eeprom_sizes(mcu_eeprom_size_map: &HashMap<u32, Vec<String>>) -> Result<(), String> {
    println!("// EEPROM sizes in bytes, generated with cube-parse");
    for size in mcu_eeprom_size_map.keys() {
        println!("#[cfg(feature = \"{}\")]", eeprom_size_to_feature(*size));
        println!("const EEPROM_SIZE_BYTES: u32 = {};", size);
    }
    Ok(())
}

fn render_pin_modes(ip: &internal_peripheral::IpGPIO) {
    let mut pin_map: HashMap<String, Vec<String>> = HashMap::new();

    for p in &ip.gpio_pin {
        let name = p.get_name();
        if let Some(n) = name {
            pin_map.insert(n, p.get_af_modes());
        }
    }

    let mut pin_map = pin_map
        .into_iter()
        .map(|(k, mut v)| {
            #[allow(clippy::redundant_closure)]
            v.sort_by(|a, b| compare_str(a, b));
            (k, v)
        })
        .collect::<Vec<_>>();

    pin_map.sort_by(|a, b| compare_str(&a.0, &b.0));

    println!("pins! {{");
    for (n, af) in pin_map {
        if af.is_empty() {
            continue;
        } else if af.len() == 1 {
            println!("    {} => {{{}}},", n, af[0]);
        } else {
            println!("    {} => {{", n);
            for a in af {
                println!("        {},", a);
            }
            println!("    }},");
        }
    }
    println!("}}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_version_to_feature() {
        // Success
        assert_eq!(
            gpio_version_to_feature("STM32L152x8_gpio_v1_0").unwrap(),
            "io-STM32L152x8"
        );
        assert_eq!(
            gpio_version_to_feature("STM32F333_gpio_v1_0").unwrap(),
            "io-STM32F333"
        );

        // Error parsing, unsupported version
        assert!(gpio_version_to_feature("STM32F333_gpio_v1_1").is_err());

        // Error parsing, wrong pattern
        assert!(gpio_version_to_feature("STM32F333_qqio_v1_0").is_err());

        // Error parsing, too many underscores
        assert!(gpio_version_to_feature("STM32_STM32F333_gpio_v1_0").is_err());
    }
}

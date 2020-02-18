use std::{collections::HashMap, env, path::Path};

use alphanumeric_sort::compare_str;
use serde_xml_rs;

mod family;
mod internal_peripheral;
mod mcu;
mod utils;

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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: ./cube-parse CUBEMX_MCU_DB_DIR MCU_FAMILY")
    }

    let db_dir = Path::new(&args[1]);

    let familys = family::Families::load(&db_dir).unwrap();

    let family = (&familys).into_iter().find(|v| v.name == args[2]).unwrap();

    let mut mcu_gpio_map = HashMap::new();

    for sf in family {
        for mcu in sf {
            let mcu_dat = mcu::Mcu::load(&db_dir, &mcu.name).unwrap();
            let gpio_version = mcu_dat.get_ip("GPIO").unwrap().get_version().to_string();
            if !mcu_gpio_map.contains_key(&gpio_version) {
                mcu_gpio_map.insert(gpio_version.clone(), Vec::new());
            }
            mcu_gpio_map
                .get_mut(&gpio_version)
                .unwrap()
                .push(mcu.name.clone());
        }
    }

    for (gpio, mcu_list) in mcu_gpio_map {
        let gpio_data = internal_peripheral::IpGPIO::load(db_dir, &gpio).unwrap();
        println!("#[cfg(any(");
        for mcu in mcu_list {
            println!("    feature = {:?},", mcu);
        }
        println!("))]");
        render_pin_modes(&gpio_data);
        println!("\n");
    }
}

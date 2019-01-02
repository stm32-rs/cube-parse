use std::{collections::HashMap, env, path::Path};

use alphanumeric_sort::compare_str;
use serde_xml_rs;

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
        eprintln!("Usage: ./cube-parse CUBEMX_MCU_DB_DIR MCU_FILE")
    }

    let db_dir = Path::new(&args[1]);



    let val = mcu::Mcu::load(&db_dir, &args[2]).unwrap();

    let gpio = {
        let gpio_name = val.get_ip("GPIO").unwrap().get_version();
        internal_peripheral::IpGPIO::load(&db_dir, gpio_name).unwrap()
    };

    render_pin_modes(&gpio);
}

use std::{error::Error, fs::File, io::BufReader, path::Path};

use serde::Deserialize;

pub fn load_file<'a, P: AsRef<Path>, Q: AsRef<Path>, R: Deserialize<'a>>(
    db_dir: P,
    file_path: Q,
) -> Result<R, Box<dyn Error>> {
    let db_dir = db_dir.as_ref();
    let mut fin = BufReader::new(File::open(&db_dir.join(file_path.as_ref()))?);

    Ok(serde_xml_rs::deserialize(&mut fin)?)
}

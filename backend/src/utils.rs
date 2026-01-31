use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn deserialize_from_file<T: DeserializeOwned + Default, P: AsRef<Path>>(path: P) -> T {
    let path = path.as_ref();
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Failed to parse {}: {}", path.display(), e);
                    T::default()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open {}: {}", path.display(), e);
            T::default()
        }
    }
}

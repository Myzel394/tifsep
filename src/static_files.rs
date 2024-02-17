pub mod static_files {
    use std::{
        fs::File,
        io::{Error, Read},
    };

    pub fn read_file_contents(path: &str) -> Result<String, Error> {
        let mut contents = String::new();

        let mut file = File::open(path)?;

        file.read_to_string(&mut contents)?;

        Ok(contents)
    }
}

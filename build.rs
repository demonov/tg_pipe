use std::{env, fs, path::Path};

fn main() -> std::io::Result<()> {
    let config = Config::new();

    config.copy_to_out_dir(".env")?;

    Ok(())
}

struct Config {
    out_dir: String,
}

impl Config {
    fn new() -> Config {
        Config {
            out_dir: env::var("OUT_DIR").expect("OUT_DIR not set"),
        }
    }

    fn copy_to_out_dir(&self, file: &str) -> std::io::Result<()> {
        let dst = Path::new(&self.out_dir).join(file);
        fs::copy(file, dst)?;
        Ok(())
    }
}
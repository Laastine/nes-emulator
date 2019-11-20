use std::fs;
use std::io::Read;

pub struct Rom {
  pub rom: Vec<u8>,
}

impl Rom {
  pub fn read_from_file(file_name: &str) -> Rom {
    let file = fs::File::open(file_name).expect("File read error");

    let rom = file
      .bytes()
      .skip(10)
      .take(16_384)
      .collect::<Result<Vec<u8>, _>>().expect("Unexpected file size");

    Rom {
      rom
    }
  }
}

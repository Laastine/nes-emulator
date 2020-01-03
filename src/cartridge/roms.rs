
#[derive(Clone)]
pub struct Roms {
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
}

impl Roms {
  pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Roms {
    Roms {
      prg_rom,
      chr_rom,
    }
  }
}

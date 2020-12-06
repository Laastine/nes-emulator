use rodio::Sink;

pub struct Apu {
  pub sink: Sink
}

impl Apu {
  pub fn new() -> Apu {
    let (_, stream_handle) = rodio::OutputStream::try_default().unwrap_or_else(|err| panic!("Stream initialization error {}", err));
    let sink= Sink::try_new(&stream_handle).unwrap_or_else(|err| panic!("Sink initialization error {}", err));
    Apu {
      sink
    }
  }
}

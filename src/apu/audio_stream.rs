use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use rodio::{OutputStreamHandle, Sink};
use rodio::buffer::SamplesBuffer;

pub struct AudioStream {
  tx: Sender<Vec<f32>>,
}

impl AudioStream {
  pub fn new() -> AudioStream {

    let (tx, rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();

    thread::spawn(move || {
      let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
      while let val = rx.try_recv() {
        if val.is_ok() {
          let new_sink = Sink::try_new(&stream_handle).unwrap();
          new_sink.append(SamplesBuffer::new(2, 44100, val.unwrap()));
          thread::sleep(Duration::from_millis(16));
          new_sink.detach();
        }
      }
    });

    AudioStream {
      tx,
    }
  }

  pub fn send_audio_buffer(&mut self, sample: Vec<f32>) {
    let _ = self.tx.send(sample);
  }
}

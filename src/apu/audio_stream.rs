use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use rodio::buffer::SamplesBuffer;
use rodio::Sink;

pub struct AudioStream {
  tx: Sender<Vec<f32>>,
}

impl AudioStream {
  pub fn new() -> AudioStream {
    let (tx, rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();

    thread::spawn(move || {
      let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
      loop {
        if let Ok(val) = rx.try_recv() {
          let new_sink = Sink::try_new(&stream_handle).unwrap();
          new_sink.append(SamplesBuffer::new(2, 44100, val));
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

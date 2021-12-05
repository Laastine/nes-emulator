use std::io::{stdout, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use crossterm::{cursor, QueueableCommand, terminal};

pub struct DebugView {
  pub tx: Sender<Vec<u8>>,
}

impl DebugView {
  pub fn new(rows: usize, cols: usize) -> DebugView {
    crossterm::execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
    let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
    let mut stdout = stdout();
    thread::spawn(move || {
      loop {
        if let Some(memory) = rx.try_iter().last() {
          let mut addr = 0;
          let mut y_ram = 2;
          for _ in 0..rows {
            stdout.queue(crossterm::style::Print(
              format!("{}{}0x{:0>4X}", cursor::MoveTo(6, y_ram), cursor::Hide, addr)
            )).unwrap();
            let mut x_ram = 2;
            for _ in 0..cols {
              stdout.queue(crossterm::style::Print(
                format!("{}{} {:0>2X}", cursor::MoveTo(x_ram, y_ram), cursor::Hide, memory[addr as usize])
              )).unwrap();
              addr += 1;
              x_ram += 3;
            }
            y_ram += 1;
          }
          let _ = stdout.flush();
        }
      }
    });
    DebugView {
      tx,
    }
  }

  pub fn send_memory_slice(&mut self, sample: Vec<u8>) {
    let _ = self.tx.send(sample);
  }
}

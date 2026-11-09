use std::io::{self, Write};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct ChannelWriter {
  sender: Sender<Vec<u8>>,
}

impl ChannelWriter {
  pub fn new(sender: Sender<Vec<u8>>) -> Self {
    Self { sender }
  }
}

impl Write for ChannelWriter {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self
      .sender
      .send(buf.to_vec())
      .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "reply channel closed"))?;
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

use std::io::Read;
use std::io::Write;

use portable_pty::Child;
use portable_pty::CommandBuilder;
use portable_pty::ExitStatus;
use portable_pty::MasterPty;
use portable_pty::PtySize;
use portable_pty::native_pty_system;

use crate::errors::pty_error::PtyError;

pub struct PtySession {
  master: Box<dyn MasterPty + Send>,
  child: Box<dyn Child + Send + Sync>,
  reader: Option<Box<dyn Read + Send>>,
  writer: Option<Box<dyn Write + Send>>,
}

impl PtySession {
  pub fn new(rows: u16, cols: u16, pixel_width: u16, pixel_height: u16) -> Result<Self, PtyError> {
    let program = default_shell();
    Self::with_command(&program, &[], rows, cols, pixel_width, pixel_height)
  }

  pub fn with_command(
    program: &str,
    args: &[&str],
    rows: u16,
    cols: u16,
    pixel_width: u16,
    pixel_height: u16,
  ) -> Result<Self, PtyError> {
    let pty_system = native_pty_system();
    let size = PtySize {
      rows,
      cols,
      pixel_width,
      pixel_height,
    };
    let pair = pty_system
      .openpty(size)
      .map_err(|err| PtyError::Open(err.to_string()))?;

    let mut command = CommandBuilder::new(program);
    command.args(args);
    let child = pair
      .slave
      .spawn_command(command)
      .map_err(|err| PtyError::Spawn(err.to_string()))?;
    drop(pair.slave);

    let reader = pair
      .master
      .try_clone_reader()
      .map_err(|err| PtyError::Reader(err.to_string()))?;
    let writer = pair
      .master
      .take_writer()
      .map_err(|err| PtyError::Writer(err.to_string()))?;

    Ok(Self {
      master: pair.master,
      child,
      reader: Some(reader),
      writer: Some(writer),
    })
  }

  pub fn take_reader(&mut self) -> Result<Box<dyn Read + Send>, PtyError> {
    self.reader.take().ok_or(PtyError::ReaderTaken)
  }

  pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, PtyError> {
    let reader = self.reader.as_mut().ok_or(PtyError::ReaderTaken)?;
    reader
      .read(buffer)
      .map_err(|err| PtyError::Read(err.to_string()))
  }

  pub fn take_writer(&mut self) -> Result<Box<dyn Write + Send>, PtyError> {
    self.writer.take().ok_or(PtyError::WriterTaken)
  }

  pub fn write(&mut self, bytes: &[u8]) -> Result<(), PtyError> {
    let writer = self.writer.as_mut().ok_or(PtyError::WriterTaken)?;
    writer
      .write_all(bytes)
      .map_err(|err| PtyError::Write(err.to_string()))?;
    writer
      .flush()
      .map_err(|err| PtyError::Write(err.to_string()))
  }

  pub fn resize(
    &self,
    rows: u16,
    cols: u16,
    pixel_width: u16,
    pixel_height: u16,
  ) -> Result<(), PtyError> {
    self
      .master
      .resize(PtySize {
        rows,
        cols,
        pixel_width,
        pixel_height,
      })
      .map_err(|err| PtyError::Resize(err.to_string()))
  }

  pub fn size(&self) -> Result<PtySize, PtyError> {
    self
      .master
      .get_size()
      .map_err(|err| PtyError::Size(err.to_string()))
  }

  pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, PtyError> {
    self
      .child
      .try_wait()
      .map_err(|err| PtyError::Wait(err.to_string()))
  }

  pub fn wait(&mut self) -> Result<ExitStatus, PtyError> {
    self
      .child
      .wait()
      .map_err(|err| PtyError::Wait(err.to_string()))
  }

  pub fn kill(&mut self) -> Result<(), PtyError> {
    self
      .child
      .kill()
      .map_err(|err| PtyError::Kill(err.to_string()))
  }

  pub fn process_id(&self) -> Option<u32> {
    self.child.process_id()
  }
}

#[cfg(unix)]
fn default_shell() -> String {
  std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

#[cfg(windows)]
fn default_shell() -> String {
  std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
}

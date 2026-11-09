use std::io::Read;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::SyncSender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use portable_pty::ExitStatus;
use wezterm_term::CursorPosition;
use wezterm_term::SemanticZone;
use wezterm_term::TerminalSize;

use crate::channels::channel_writer::ChannelWriter;
use crate::channels::model_command::ModelCommand;
use crate::errors::session_error::SessionError;
use crate::models::terminal_model::TerminalModel;
use crate::sessions::pty_session::PtySession;

pub struct Session {
  pty: Option<PtySession>,
  commands: Option<SyncSender<ModelCommand>>,
  input: Option<Sender<Vec<u8>>>,
  model_thread: Option<JoinHandle<()>>,
  reader_thread: Option<JoinHandle<()>>,
  writer_thread: Option<JoinHandle<()>>,
}

impl Session {
  pub fn new(mut pty: PtySession, scrollback: usize) -> Result<Self, SessionError> {
    let size = pty.size()?;
    let reader = pty.take_reader()?;
    let pty_writer = pty.take_writer()?;

    let (input, input_receiver) = mpsc::channel::<Vec<u8>>();
    let reply_writer = ChannelWriter::new(input.clone());
    let model = TerminalModel::new(
      size.rows as usize,
      size.cols as usize,
      size.pixel_width as usize,
      size.pixel_height as usize,
      scrollback,
      Box::new(reply_writer),
    );

    let (commands, command_receiver) = mpsc::sync_channel::<ModelCommand>(command_queue_capacity());
    let reader_commands = commands.clone();

    let model_thread = thread::spawn(move || run_model(model, command_receiver));
    let reader_thread = thread::spawn(move || run_reader(reader, reader_commands));
    let writer_thread = thread::spawn(move || run_writer(pty_writer, input_receiver));

    Ok(Self {
      pty: Some(pty),
      commands: Some(commands),
      input: Some(input),
      model_thread: Some(model_thread),
      reader_thread: Some(reader_thread),
      writer_thread: Some(writer_thread),
    })
  }

  pub fn write(&self, bytes: &[u8]) -> Result<(), SessionError> {
    let input = self.input.as_ref().ok_or(SessionError::Closed)?;
    input.send(bytes.to_vec()).map_err(|_| SessionError::Closed)
  }

  pub fn resize(
    &self,
    rows: u16,
    cols: u16,
    pixel_width: u16,
    pixel_height: u16,
  ) -> Result<(), SessionError> {
    let pty = self.pty.as_ref().ok_or(SessionError::Closed)?;
    pty.resize(rows, cols, pixel_width, pixel_height)?;
    self.send(ModelCommand::Resize {
      rows: rows as usize,
      cols: cols as usize,
      pixel_width: pixel_width as usize,
      pixel_height: pixel_height as usize,
    })
  }

  pub fn screen_text(&self) -> Result<String, SessionError> {
    self.query(ModelCommand::ScreenText)
  }

  pub fn line_text(&self, row: usize) -> Result<String, SessionError> {
    self.query(|reply| ModelCommand::LineText { row, reply })
  }

  pub fn dimensions(&self) -> Result<TerminalSize, SessionError> {
    self.query(ModelCommand::Dimensions)
  }

  pub fn cursor_position(&self) -> Result<CursorPosition, SessionError> {
    self.query(ModelCommand::CursorPosition)
  }

  pub fn semantic_zones(&self) -> Result<Vec<SemanticZone>, SessionError> {
    self
      .query(ModelCommand::SemanticZones)?
      .map_err(|err| SessionError::Query(err.to_string()))
  }

  pub fn current_dir(&self) -> Result<Option<String>, SessionError> {
    self.query(ModelCommand::CurrentDir)
  }

  pub fn image_cell_count(&self) -> Result<usize, SessionError> {
    self.query(ModelCommand::ImageCellCount)
  }

  pub fn scrollback_size(&self) -> Result<usize, SessionError> {
    self.query(ModelCommand::ScrollbackSize)
  }

  pub fn wait_until_screen_contains(
    &self,
    needle: &str,
    timeout: Duration,
  ) -> Result<bool, SessionError> {
    let deadline = Instant::now() + timeout;
    loop {
      if self.screen_text()?.contains(needle) {
        return Ok(true);
      }
      if Instant::now() >= deadline {
        return Ok(false);
      }
      thread::sleep(poll_interval());
    }
  }

  pub fn shutdown(&mut self) -> Result<Option<ExitStatus>, SessionError> {
    let status = match self.pty.take() {
      Some(mut pty) => {
        let _ = pty.kill();
        pty.wait().ok()
      }
      None => None,
    };

    if let Some(commands) = self.commands.take() {
      let _ = commands.send(ModelCommand::Shutdown);
    }
    self.input.take();
    self.join_threads();

    Ok(status)
  }

  fn send(&self, command: ModelCommand) -> Result<(), SessionError> {
    let commands = self.commands.as_ref().ok_or(SessionError::Closed)?;
    commands
      .send(command)
      .map_err(|_| SessionError::ModelStopped)
  }

  fn query<T>(&self, make: impl FnOnce(Sender<T>) -> ModelCommand) -> Result<T, SessionError> {
    let (reply, receiver) = mpsc::channel();
    self.send(make(reply))?;
    receiver
      .recv_timeout(query_timeout())
      .map_err(|err| match err {
        mpsc::RecvTimeoutError::Timeout => SessionError::Timeout,
        mpsc::RecvTimeoutError::Disconnected => SessionError::ModelStopped,
      })
  }

  fn join_threads(&mut self) {
    if let Some(handle) = self.reader_thread.take() {
      let _ = handle.join();
    }
    if let Some(handle) = self.model_thread.take() {
      let _ = handle.join();
    }
    if let Some(handle) = self.writer_thread.take() {
      let _ = handle.join();
    }
  }
}

impl Drop for Session {
  fn drop(&mut self) {
    let _ = self.shutdown();
  }
}

fn run_model(mut model: TerminalModel, commands: Receiver<ModelCommand>) {
  while let Ok(command) = commands.recv() {
    match command {
      ModelCommand::Advance(bytes) => model.advance_bytes(&bytes),
      ModelCommand::Resize {
        rows,
        cols,
        pixel_width,
        pixel_height,
      } => model.resize(rows, cols, pixel_width, pixel_height),
      ModelCommand::ScreenText(reply) => {
        let _ = reply.send(model.screen_text());
      }
      ModelCommand::LineText { row, reply } => {
        let _ = reply.send(model.line_text(row));
      }
      ModelCommand::Dimensions(reply) => {
        let _ = reply.send(model.dimensions());
      }
      ModelCommand::CursorPosition(reply) => {
        let _ = reply.send(model.cursor_position());
      }
      ModelCommand::SemanticZones(reply) => {
        let _ = reply.send(model.semantic_zones());
      }
      ModelCommand::CurrentDir(reply) => {
        let _ = reply.send(model.current_dir());
      }
      ModelCommand::ImageCellCount(reply) => {
        let _ = reply.send(model.image_cell_count());
      }
      ModelCommand::ScrollbackSize(reply) => {
        let _ = reply.send(model.scrollback_size());
      }
      ModelCommand::Shutdown => break,
    }
  }
}

fn run_reader(mut reader: Box<dyn Read + Send>, commands: SyncSender<ModelCommand>) {
  let mut buffer = vec![0u8; read_buffer_size()];
  loop {
    match reader.read(&mut buffer) {
      Ok(0) => break,
      Ok(read) => {
        let chunk = buffer[..read].to_vec();
        if commands.send(ModelCommand::Advance(chunk)).is_err() {
          break;
        }
      }
      Err(_) => break,
    }
  }
}

fn run_writer(mut writer: Box<dyn Write + Send>, input: Receiver<Vec<u8>>) {
  while let Ok(bytes) = input.recv() {
    if writer.write_all(&bytes).is_err() {
      break;
    }
    if writer.flush().is_err() {
      break;
    }
  }
}

fn command_queue_capacity() -> usize {
  64
}

fn read_buffer_size() -> usize {
  8192
}

fn poll_interval() -> Duration {
  Duration::from_millis(5)
}

fn query_timeout() -> Duration {
  Duration::from_secs(5)
}

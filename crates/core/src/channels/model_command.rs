use std::sync::mpsc::Sender;

use wezterm_term::CursorPosition;
use wezterm_term::SemanticZone;
use wezterm_term::TerminalSize;

use crate::errors::model_error::ModelError;

pub enum ModelCommand {
  Advance(Vec<u8>),
  Resize {
    rows: usize,
    cols: usize,
    pixel_width: usize,
    pixel_height: usize,
  },
  ScreenText(Sender<String>),
  LineText {
    row: usize,
    reply: Sender<String>,
  },
  Dimensions(Sender<TerminalSize>),
  CursorPosition(Sender<CursorPosition>),
  SemanticZones(Sender<Result<Vec<SemanticZone>, ModelError>>),
  CurrentDir(Sender<Option<String>>),
  ImageCellCount(Sender<usize>),
  ScrollbackSize(Sender<usize>),
  Shutdown,
}

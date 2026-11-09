use std::io::Write;
use std::sync::Arc;

use wezterm_term::CursorPosition;
use wezterm_term::Line;
use wezterm_term::SemanticZone;
use wezterm_term::Terminal;
use wezterm_term::TerminalSize;

use crate::configs::model_config::ModelConfig;
use crate::errors::model_error::ModelError;

pub struct TerminalModel {
  terminal: Terminal,
}

impl TerminalModel {
  pub fn new(
    rows: usize,
    cols: usize,
    pixel_width: usize,
    pixel_height: usize,
    scrollback: usize,
    writer: Box<dyn Write + Send>,
  ) -> Self {
    let config = Arc::new(ModelConfig::new(scrollback));
    let size = TerminalSize {
      rows,
      cols,
      pixel_width,
      pixel_height,
      dpi: 96,
    };
    let terminal = Terminal::new(
      size,
      config,
      "pi-terminal",
      env!("CARGO_PKG_VERSION"),
      writer,
    );
    Self { terminal }
  }

  pub fn advance_bytes(&mut self, bytes: &[u8]) {
    self.terminal.advance_bytes(bytes);
  }

  pub fn resize(&mut self, rows: usize, cols: usize, pixel_width: usize, pixel_height: usize) {
    let dpi = self.terminal.get_size().dpi;
    self.terminal.resize(TerminalSize {
      rows,
      cols,
      pixel_width,
      pixel_height,
      dpi,
    });
  }

  pub fn line_text(&self, row: usize) -> String {
    let lines = self.visible_lines();
    lines
      .get(row)
      .map(|line| line.as_str().to_string())
      .unwrap_or_default()
  }

  pub fn screen_text(&self) -> String {
    let lines = self.visible_lines();
    lines
      .iter()
      .map(|line| line.as_str())
      .collect::<Vec<_>>()
      .join("\n")
  }

  pub fn dimensions(&self) -> TerminalSize {
    self.terminal.get_size()
  }

  pub fn cursor_position(&self) -> CursorPosition {
    self.terminal.cursor_pos()
  }

  pub fn semantic_zones(&mut self) -> Result<Vec<SemanticZone>, ModelError> {
    self
      .terminal
      .get_semantic_zones()
      .map_err(|err| ModelError::SemanticZones(err.to_string()))
  }

  pub fn current_dir(&self) -> Option<String> {
    self.terminal.get_current_dir().map(|url| url.to_string())
  }

  pub fn image_cell_count(&self) -> usize {
    self
      .visible_lines()
      .iter()
      .map(|line| {
        line
          .visible_cells()
          .map(|cell| {
            cell
              .attrs()
              .images()
              .map(|images| images.len())
              .unwrap_or(0)
          })
          .sum::<usize>()
      })
      .sum()
  }

  pub fn scrollback_size(&self) -> usize {
    self.terminal.get_config().scrollback_size()
  }

  fn visible_lines(&self) -> Vec<Line> {
    let screen = self.terminal.screen();
    let total = screen.scrollback_rows();
    let start = total.saturating_sub(screen.physical_rows);
    screen.lines_in_phys_range(start..total)
  }
}

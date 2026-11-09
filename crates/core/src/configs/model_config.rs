use wezterm_term::TerminalConfiguration;
use wezterm_term::color::ColorPalette;

#[derive(Debug)]
pub struct ModelConfig {
  scrollback: usize,
}

impl ModelConfig {
  pub fn new(scrollback: usize) -> Self {
    Self { scrollback }
  }
}

impl TerminalConfiguration for ModelConfig {
  fn scrollback_size(&self) -> usize {
    self.scrollback
  }

  fn color_palette(&self) -> ColorPalette {
    ColorPalette::default()
  }
}

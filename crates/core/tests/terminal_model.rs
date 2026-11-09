use std::time::Instant;

use pi_core::models::terminal_model::TerminalModel;

mod helpers;

use helpers::text_payload;

fn model() -> TerminalModel {
  TerminalModel::new(24, 80, 800, 600, 1000, Box::new(std::io::sink()))
}

#[test]
fn resize_changes_dimensions() {
  let mut model = model();
  let initial = model.dimensions();
  assert_eq!(initial.rows, 24);
  assert_eq!(initial.cols, 80);

  model.resize(40, 120, 1200, 900);

  let resized = model.dimensions();
  assert_eq!(resized.rows, 40);
  assert_eq!(resized.cols, 120);
  assert_eq!(resized.pixel_width, 1200);
  assert_eq!(resized.pixel_height, 900);
}

#[test]
fn osc133_records_semantic_zones() {
  let mut model = model();

  model.advance_bytes(b"\x1b]133;A\x07pi> ");
  model.advance_bytes(b"\x1b]133;B\x07");
  model.advance_bytes(b"echo hi\n");
  model.advance_bytes(b"\x1b]133;C\x07hi\n");
  model.advance_bytes(b"\x1b]133;D;0\x07");

  let zones = model.semantic_zones().expect("semantic zones");

  assert!(zones.len() >= 3, "zones: {zones:?}");

  let kinds: Vec<String> = zones
    .iter()
    .map(|zone| format!("{:?}", zone.semantic_type))
    .collect();

  assert!(kinds.iter().any(|kind| kind == "Prompt"), "{kinds:?}");
  assert!(kinds.iter().any(|kind| kind == "Input"), "{kinds:?}");
  assert!(kinds.iter().any(|kind| kind == "Output"), "{kinds:?}");
}

#[test]
fn osc7_reports_current_directory() {
  let mut model = model();

  model.advance_bytes(b"\x1b]7;file:///tmp\x07");

  let cwd = model.current_dir().expect("cwd from OSC 7");
  assert!(cwd.contains("/tmp"), "cwd: {cwd}");
}

#[test]
fn images_are_stored_when_engine_supports_them() {
  let mut model = model();

  let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
  let iterm = format!("\x1b]1337;File=inline=1;width=2;height=2:{png}\x07");
  model.advance_bytes(iterm.as_bytes());
  let after_iterm = model.image_cell_count();

  model.advance_bytes(b"\x1bP0;0;0q\"1;1;4;4#0;2;100;0;0#0@@@@\x1b\\");
  let after_sixel = model.image_cell_count();

  println!("image cells after iTerm2 File: {after_iterm}");
  println!("image cells after sixel: {after_sixel}");

  assert!(after_iterm >= 1, "iTerm2 File image was not stored");
  assert!(after_sixel > after_iterm, "sixel image was not stored");
}

#[test]
fn throughput_smoke() {
  let mut model = model();
  let content = text_payload::payload(2 * 1024 * 1024);

  let mut pushed = 0;
  for chunk in content.chunks(8192) {
    model.advance_bytes(chunk);
    pushed += chunk.len();
  }

  assert_eq!(pushed, content.len());
  assert!(!model.screen_text().is_empty());
}

#[test]
#[ignore = "benchmark: run with cargo test --release -p pi-core -- --ignored throughput_benchmark --nocapture"]
fn throughput_benchmark() {
  let mut model = model();
  let content = text_payload::payload(50 * 1024 * 1024);

  let start = Instant::now();
  for chunk in content.chunks(8192) {
    model.advance_bytes(chunk);
  }
  let elapsed = start.elapsed();

  let megabytes = content.len() as f64 / 1_000_000.0;
  let rate = megabytes / elapsed.as_secs_f64();
  println!("throughput_benchmark: {rate:.2} MB/s over {elapsed:?}");
}

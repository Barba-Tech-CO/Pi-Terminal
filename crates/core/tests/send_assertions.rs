use pi_core::channels::channel_writer::ChannelWriter;
use pi_core::channels::model_command::ModelCommand;
use pi_core::models::terminal_model::TerminalModel;
use pi_core::sessions::session::Session;

fn assert_send<T: Send>() {}

#[test]
fn types_handed_to_threads_are_send() {
  assert_send::<ChannelWriter>();
  assert_send::<ModelCommand>();
  assert_send::<TerminalModel>();
  assert_send::<Session>();
}

#[test]
fn inner_terminal_is_send() {
  assert_send::<wezterm_term::Terminal>();

  let model = TerminalModel::new(24, 80, 800, 600, 1000, Box::new(std::io::sink()));
  let handle = std::thread::spawn(move || {
    let mut model = model;
    model.advance_bytes(b"send-check");
    model.screen_text()
  });

  let text = handle.join().expect("model thread");
  assert!(text.contains("send-check"), "text: {text}");
}

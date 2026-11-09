use std::time::Duration;

use pi_core::sessions::pty_session::PtySession;
use pi_core::sessions::session::Session;

#[cfg(unix)]
fn printf_pty() -> PtySession {
  PtySession::with_command("sh", &["-c", "printf hello"], 24, 80, 800, 600).expect("pty")
}

#[cfg(windows)]
fn printf_pty() -> PtySession {
  PtySession::with_command("cmd.exe", &["/c", "echo hello"], 24, 80, 800, 600).expect("pty")
}

#[test]
fn command_output_reaches_screen() {
  let pty = printf_pty();
  let mut session = Session::new(pty, 1000).expect("session");

  let seen = session
    .wait_until_screen_contains("hello", Duration::from_secs(10))
    .expect("wait");
  assert!(seen);

  let status = session.shutdown().expect("shutdown").expect("status");
  assert!(status.success());
}

#[test]
fn resize_updates_pty_and_model() {
  let pty = printf_pty();
  let mut session = Session::new(pty, 1000).expect("session");

  session.resize(40, 120, 1200, 900).expect("resize");

  let size = session.dimensions().expect("dimensions");
  assert_eq!(size.rows, 40);
  assert_eq!(size.cols, 120);

  session.shutdown().expect("shutdown");
}

#[test]
fn write_sends_input_to_child() {
  #[cfg(unix)]
  let pty = PtySession::with_command("sh", &[], 24, 80, 800, 600).expect("pty");
  #[cfg(windows)]
  let pty = PtySession::with_command("cmd.exe", &[], 24, 80, 800, 600).expect("pty");

  let mut session = Session::new(pty, 1000).expect("session");
  session.write(b"echo written-marker\n").expect("write");

  let seen = session
    .wait_until_screen_contains("written-marker", Duration::from_secs(10))
    .expect("wait");
  assert!(seen);

  session.shutdown().expect("shutdown");
}

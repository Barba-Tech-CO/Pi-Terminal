pub fn payload(target_bytes: usize) -> Vec<u8> {
  let mut payload = Vec::with_capacity(target_bytes + 80);
  let mut index = 0usize;
  while payload.len() < target_bytes {
    let mut body = format!("line {index:06} the quick brown fox jumps over the lazy dog");
    while body.len() < 78 {
      body.push(' ');
    }
    payload.extend_from_slice(body.as_bytes());
    payload.extend_from_slice(b"\r\n");
    index += 1;
  }
  payload
}

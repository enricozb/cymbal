#[macro_export]
macro_rules! color {
  ($text:expr, black) => {
    concat!("\x1b[30m", $text, "\x1b[0m")
  };
  ($text:expr, red) => {
    concat!("\x1b[31m", $text, "\x1b[0m")
  };
  ($text:expr, green) => {
    concat!("\x1b[32m", $text, "\x1b[0m")
  };
  ($text:expr, yellow) => {
    concat!("\x1b[33m", $text, "\x1b[0m")
  };
  ($text:expr, blue) => {
    concat!("\x1b[34m", $text, "\x1b[0m")
  };
  ($text:expr, magenta) => {
    concat!("\x1b[35m", $text, "\x1b[0m")
  };
  ($text:expr, cyan) => {
    concat!("\x1b[36m", $text, "\x1b[0m")
  };
  ($text:expr, white) => {
    concat!("\x1b[37m", $text, "\x1b[0m")
  };
  ($text:expr, bright_black) => {
    concat!("\x1b[90m", $text, "\x1b[0m")
  };
  ($text:expr, bright_red) => {
    concat!("\x1b[91m", $text, "\x1b[0m")
  };
  ($text:expr, bright_green) => {
    concat!("\x1b[92m", $text, "\x1b[0m")
  };
  ($text:expr, bright_yellow) => {
    concat!("\x1b[93m", $text, "\x1b[0m")
  };
  ($text:expr, bright_blue) => {
    concat!("\x1b[94m", $text, "\x1b[0m")
  };
  ($text:expr, bright_magenta) => {
    concat!("\x1b[95m", $text, "\x1b[0m")
  };
  ($text:expr, bright_cyan) => {
    concat!("\x1b[96m", $text, "\x1b[0m")
  };
  ($text:expr, bright_white) => {
    concat!("\x1b[97m", $text, "\x1b[0m")
  };
}

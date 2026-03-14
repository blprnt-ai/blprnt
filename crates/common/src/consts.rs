use line_ending::LineEnding;

#[cfg(not(windows))]
pub const OS_LINE_ENDING: LineEnding = LineEnding::LF;

#[cfg(windows)]
pub const OS_LINE_ENDING: LineEnding = LineEnding::CRLF;

pub const SURREAL_DB_PORT: u16 = 14145;

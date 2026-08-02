//! Terminal ANSI formatting and color code constants.

/// Reset all formatting.
pub const ANSI_COLOR_RESET: &str = "\x1b[0m";

/// Enable bold font and red text color.
pub const ANSI_COLOR_BOLD_RED: &str = "\x1b[1;31m";

/// Enable bold font and green text color.
pub const ANSI_COLOR_BOLD_GREEN: &str = "\x1b[1;32m";

/// Enable bold font and yellow text color.
pub const ANSI_COLOR_BOLD_YELLOW: &str = "\x1b[1;33m";

/// Enable bold font and blue text color.
pub const ANSI_COLOR_BOLD_BLUE: &str = "\x1b[1;34m";

/// Enable bold font and magenta text color.
pub const ANSI_COLOR_BOLD_MAGENTA: &str = "\x1b[1;35m";

/// Enable bold font and cyan text color.
pub const ANSI_COLOR_BOLD_CYAN: &str = "\x1b[1;36m";

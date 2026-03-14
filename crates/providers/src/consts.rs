pub const MAX_BACKOFF_SHIFT: u32 = 10;
pub const JITTER_DIVISOR: u64 = 2;
pub const JITTER_MIN_ADD: u64 = 1;

pub const HTTP_AUTH_1: u16 = 401;
pub const HTTP_AUTH_2: u16 = 403;
pub const HTTP_TIMEOUT: u16 = 408;
pub const HTTP_RATE_LIMIT: u16 = 429;
pub const HTTP_BAD_REQUEST: u16 = 400;
pub const HTTP_CANCELED: u16 = 499;
pub const HTTP_MIN_UPSTREAM: u16 = 500;
pub const HTTP_MAX_UPSTREAM: u16 = 599;
pub const HTTP_NOT_SUPPORTED_LIST: [u16; 4] = [404, 405, 415, 422];

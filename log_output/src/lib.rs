pub mod generator;
pub mod logger;
pub mod runner;

pub use generator::generate_string;
pub use logger::log_entry;
pub use runner::run;

pub const INTERVAL_SECS: u64 = 5;

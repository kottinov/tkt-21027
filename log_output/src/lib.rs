pub mod generator;
pub mod logger;
pub mod runner;
pub mod server;

pub use generator::generate_string;
pub use logger::log_entry;
pub use runner::start_logger;
pub use server::run;

pub const INTERVAL_SECS: u64 = 5;

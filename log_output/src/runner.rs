use crate::{INTERVAL_SECS, logger};
use std::{thread, time::Duration};

pub fn run(value: &str) -> ! {
    let _span = tracing::info_span!("log_loop", instance_id = %value).entered();

    loop {
        logger::log_entry(value);
        thread::sleep(Duration::from_secs(INTERVAL_SECS));
    }
}

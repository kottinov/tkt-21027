use crate::{logger, INTERVAL_SECS};
use std::{thread, time::Duration};

pub fn start_logger(value: String) {
    thread::spawn(move || {
        let _span = tracing::info_span!("log_loop", instance_id = %value).entered();

        loop {
            logger::log_entry(&value);
            thread::sleep(Duration::from_secs(INTERVAL_SECS));
        }
    });
}

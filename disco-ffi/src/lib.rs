use core::ffi::c_int;

use paper_rs::LoggerFnPtr;
use tracing::info;

#[unsafe(no_mangle)]
pub extern "C" fn disco_init(log_upcall: LoggerFnPtr) {
    paper_rs::install_upcall(log_upcall);
    paper_rs::install_subscriber();
    info!("disco_init called");
}

#[unsafe(no_mangle)]
pub extern "C" fn disco_ping() -> c_int {
    info!("disco_ping called");
    42
}

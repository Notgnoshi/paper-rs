use paper_rs::LogUpcall;
use tracing::info;

#[unsafe(no_mangle)]
pub extern "C" fn disco_init(log_upcall: LogUpcall) {
    paper_rs::install_upcall(log_upcall);
    paper_rs::install_subscriber();
    info!("disco_init called");
}

#[unsafe(no_mangle)]
pub extern "C" fn disco_ping() -> i32 {
    info!("disco_ping called");
    42
}

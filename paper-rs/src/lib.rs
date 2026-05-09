pub fn install_subscriber() {
    let _ = tracing_subscriber::fmt().try_init();
}

/// Build the greeting reply for the /hello command.
pub fn hello(name: &str) -> String {
    tracing::debug!("Greeting {name}");
    format!("Hello, {name}!")
}

/// Pick a DyeColor index (0..=15) for a sheep, varying per-click and per-sheep so
/// rapid clicks cycle through colors.
pub fn pick_sheep_color(uuid: [u8; 16]) -> u8 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let uuid_sum: u32 = uuid.iter().map(|&b| b as u32).sum();
    ((nanos ^ uuid_sum) % 16) as u8
}

/// Build the greeting reply for the /hello command.
pub fn hello(name: &str) -> String {
    tracing::info!("Greeting {name}");
    format!("Hello, {name}!")
}

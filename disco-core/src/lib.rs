use jni::sys::{JNIEnv, jobject};
use paper::bukkit::event::{PlayerInteractEntityEvent, PlayerInteractEntityEventRef};
use paper::bukkit::{CommandSender, DyeColor, Sheep};
use paper::{Api, CoreApi, PluginBuilder};

#[unsafe(no_mangle)]
pub extern "C" fn paper_core_init(env: *mut JNIEnv, plugin: jobject) -> *const CoreApi {
    paper::core_init(env, plugin, |b: &mut PluginBuilder| {
        b.on::<PlayerInteractEntityEvent>(handle_interact);
        b.command("hello", handle_hello);
    })
}

fn handle_interact<'l>(api: &mut Api<'_, 'l>, event: &PlayerInteractEntityEventRef<'l>) {
    let entity = match event.right_clicked(api) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("right_clicked failed: {e}");
            return;
        }
    };
    if let Some(mut sheep) = entity.cast::<Sheep>(api) {
        tracing::debug!("Recoloring a sheep to pink");
        if let Err(e) = sheep.set_color(api, DyeColor::Pink) {
            tracing::warn!("set_color failed: {e}");
        }
    }
}

/// Build the greeting reply for the /hello command.
pub fn hello(name: &str) -> String {
    tracing::debug!("Greeting {name}");
    format!("<green>Hello, <yellow>{name}</yellow>!")
}

fn handle_hello(api: &mut Api, sender: &CommandSender, args: &[String]) -> bool {
    let name = match args.first() {
        Some(arg) => arg.clone(),
        None => match sender.name(api) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!("getName failed: {e}");
                return false;
            }
        },
    };
    let reply = hello(&name);
    if let Err(e) = sender.send_message(api, reply) {
        tracing::warn!("sendMessage failed: {e}");
        return false;
    }
    true
}

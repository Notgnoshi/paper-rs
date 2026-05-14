use jni::sys::{JNIEnv, jobject};
use papermc::bukkit::dialog::{
    ActionButton, ClickCallbackOptions, Dialog, DialogAction, DialogBase, DialogType,
};
use papermc::bukkit::event::{
    EntityDamageByEntityEvent, EntityDamageByEntityEventRef, PlayerInteractEntityEvent,
    PlayerInteractEntityEventRef,
};
use papermc::bukkit::{Audience, CommandSender, CommandSenderInst, Component, DyeColor, Sheep};
use papermc::{Api, FnTable, PluginBuilder};

#[unsafe(no_mangle)]
pub extern "C" fn papermc_plugin_init(env: *mut JNIEnv, plugin: jobject) -> *const FnTable {
    papermc::plugin_init(env, plugin, |b: &mut PluginBuilder| {
        b.on::<PlayerInteractEntityEvent>(handle_interact)?;
        b.on::<EntityDamageByEntityEvent>(handle_sheep_damaged)?;
        b.command("hello", handle_hello)?;
        Ok(())
    })
}

fn handle_sheep_damaged<'l>(api: &mut Api<'_, 'l>, event: &EntityDamageByEntityEventRef<'l>) {
    let entity = match event.entity(api) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("getEntity failed: {e}");
            return;
        }
    };
    if entity.cast::<Sheep>(api).is_none() {
        return;
    }
    let player = match event.player_attacker(api) {
        Ok(Some(p)) => p,
        Ok(None) => return,
        Err(e) => {
            tracing::warn!("player_attacker failed: {e}");
            return;
        }
    };
    let dialog = match build_baaa_dialog(api) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("build_baaa_dialog failed: {e}");
            return;
        }
    };
    if let Err(e) = player.show_dialog(api, &dialog) {
        tracing::warn!("show_dialog failed: {e}");
    }
}

fn build_baaa_dialog<'l>(api: &mut Api<'_, 'l>) -> eyre::Result<Dialog<'l>> {
    let title = Component::mini_message(api, "<red>BAAAA?!</red>")?;
    let base = DialogBase::builder(api, &title)?.build(api)?;

    let options = ClickCallbackOptions::builder(api)?
        .uses(api, 1)?
        .build(api)?;

    let label_quiet = Component::mini_message(api, "Baaa.")?;
    let label_loud = Component::mini_message(api, "BAAA!")?;

    let action_quiet = DialogAction::custom_click_callback(api, &options, |_api, _view, _aud| {
        tracing::info!("sheep said: Baaa.");
    })?;
    let options_loud = ClickCallbackOptions::builder(api)?
        .uses(api, 1)?
        .build(api)?;
    let action_loud =
        DialogAction::custom_click_callback(api, &options_loud, |_api, _view, _aud| {
            tracing::info!("sheep said: BAAA!");
        })?;

    let btn_quiet = ActionButton::create(api, &label_quiet, None, 150, Some(&action_quiet))?;
    let btn_loud = ActionButton::create(api, &label_loud, None, 150, Some(&action_loud))?;

    let dtype = DialogType::multi_action(api, &[btn_quiet, btn_loud])?;
    Dialog::create(api, &base, &dtype)
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

fn handle_hello(api: &mut Api, sender: &CommandSenderInst, args: &[String]) -> bool {
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

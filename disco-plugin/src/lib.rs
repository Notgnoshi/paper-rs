use jni::sys::{JNIEnv, jobject};
use papermc::bukkit::dialog::{
    ActionButton, ClickCallbackOptions, Dialog, DialogAction, DialogBase, DialogType,
};
use papermc::bukkit::event::{
    EntityDamageByEntityEvent, EntityDamageByEntityEventRef, PlayerInteractEntityEvent,
    PlayerInteractEntityEventRef,
};
use papermc::bukkit::{Audience, CommandSender, CommandSenderInst, Component, DyeColor, Sheep};
use papermc::{Api, FnTable, Plugin, SetupApi};

pub struct DiscoPlugin;

impl Plugin for DiscoPlugin {
    fn on_enable(api: &mut SetupApi<'_, '_, Self>) -> eyre::Result<Self> {
        api.register_event::<PlayerInteractEntityEvent, _>(Self::handle_interact)?;
        api.register_event::<EntityDamageByEntityEvent, _>(Self::handle_sheep_damaged)?;
        api.register_command("hello", Self::handle_hello)?;
        Ok(DiscoPlugin)
    }
}

impl DiscoPlugin {
    fn handle_interact<'l>(
        &mut self,
        api: &mut Api<'_, 'l>,
        event: &PlayerInteractEntityEventRef<'l>,
    ) -> eyre::Result<()> {
        let entity = event.right_clicked(api)?;
        if let Some(mut sheep) = entity.cast::<Sheep>(api) {
            tracing::debug!("Recoloring a sheep to pink");
            sheep.set_color(api, DyeColor::Pink)?;
        }
        Ok(())
    }

    fn handle_sheep_damaged<'l>(
        &mut self,
        api: &mut Api<'_, 'l>,
        event: &EntityDamageByEntityEventRef<'l>,
    ) -> eyre::Result<()> {
        let entity = event.entity(api)?;
        if entity.cast::<Sheep>(api).is_none() {
            return Ok(());
        }
        let Some(player) = event.player_attacker(api)? else {
            return Ok(());
        };
        let dialog = build_baaa_dialog(api)?;
        player.show_dialog(api, &dialog)?;
        Ok(())
    }

    fn handle_hello(
        &mut self,
        api: &mut Api<'_, '_>,
        sender: &CommandSenderInst<'_>,
        args: &[String],
    ) -> eyre::Result<bool> {
        let name = match args.first() {
            Some(arg) => arg.clone(),
            None => sender.name(api)?,
        };
        tracing::debug!("Greeting {name}");
        sender.send_message(api, format!("<green>Hello, <yellow>{name}</yellow>!"))?;
        Ok(true)
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

#[unsafe(no_mangle)]
pub extern "C" fn papermc_plugin_init(env: *mut JNIEnv, plugin: jobject) -> *const FnTable {
    papermc::init::<DiscoPlugin>(env, plugin)
}

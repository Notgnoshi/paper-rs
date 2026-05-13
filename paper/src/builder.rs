use jni::Env;
use jni::objects::JObject;

use crate::api::Api;
use crate::bukkit::CommandSenderInst;
use crate::bukkit::event::Event;
use crate::{ctx, registration};

/// Best-effort lookup of `obj.getClass().getName()` for diagnostic logging.
///
/// Returns `<unknown>` on any JNI failure and clears the resulting exception so the caller's
/// subsequent JNI calls aren't poisoned.
fn actual_class_name(env: &mut Env<'_>, obj: &JObject<'_>) -> String {
    match (|| -> jni::errors::Result<String> {
        let class = env.get_object_class(obj)?;
        let name_jstr = class.get_name(env)?;
        name_jstr.try_to_string(env)
    })() {
        Ok(s) => s,
        Err(_) => {
            env.exception_clear();
            "<unknown>".to_string()
        }
    }
}

pub struct PluginBuilder<'a, 'local> {
    pub(crate) env: &'a mut Env<'local>,
}

impl<'a, 'local> PluginBuilder<'a, 'local> {
    pub(crate) fn new(env: &'a mut Env<'local>) -> Self {
        Self { env }
    }

    /// Register a Bukkit event handler.
    ///
    /// The event type is identified by an implementation of [`Event`] (typically a marker type in
    /// `paper::bukkit::event`); the handler receives the corresponding `Event::Wrapper` for the JNI
    /// frame's lifetime.
    ///
    /// Returns `Err` if registration with Bukkit fails. Callers in `paper_core_init` should
    /// propagate via `?` so a failed registration aborts plugin init cleanly with the underlying
    /// Java exception preserved.
    pub fn on<E: Event>(
        &mut self,
        handler: impl for<'b, 'l> Fn(&mut Api<'b, 'l>, &E::Wrapper<'l>) + Send + Sync + 'static,
    ) -> jni::errors::Result<()> {
        let id = ctx::with_ctx(|c| {
            let id = c.next_handler_id();
            c.event_handlers.insert(
                id,
                Box::new(move |env, obj| match E::wrap(env, obj) {
                    Ok(wrapper) => {
                        let mut api = Api::new(env);
                        handler(&mut api, wrapper);
                    }
                    Err(jni::errors::Error::WrongObjectType) => {
                        // Bukkit subclasses that don't declare their own static HandlerList share
                        // the parent class's list, so fires of sibling/parent events get routed
                        // here.
                        tracing::debug!(
                            "event skipped: expected {:?}, actual {}",
                            E::CLASS_NAME.as_cstr(),
                            actual_class_name(env, obj),
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "event dispatch type-check failed for {:?}: {e}",
                            E::CLASS_NAME
                        );
                    }
                }),
            );
            id
        })
        .expect("Ctx installed during core_init");
        registration::subscribe_event(self.env, E::CLASS_NAME, id)
    }

    /// Register a Bukkit command handler under `name`.
    ///
    /// Returns true from the handler to indicate the command was handled, false to let Bukkit print
    /// usage.
    ///
    /// Returns `Err` if registration fails; see [`Self::on`] for the propagation pattern.
    pub fn command(
        &mut self,
        name: &str,
        handler: impl for<'b, 'l> Fn(&mut Api<'b, 'l>, &CommandSenderInst<'l>, &[String]) -> bool
        + Send
        + Sync
        + 'static,
    ) -> jni::errors::Result<()> {
        let id = ctx::with_ctx(|c| {
            let id = c.next_handler_id();
            c.command_handlers.insert(
                id,
                Box::new(move |env, sender_obj, args| {
                    match CommandSenderInst::wrap_ref(env, sender_obj) {
                        Ok(sender) => {
                            let mut api = Api::new(env);
                            handler(&mut api, sender, args)
                        }
                        Err(e) => {
                            tracing::warn!("command dispatch type-check failed: {e}");
                            false
                        }
                    }
                }),
            );
            id
        })
        .expect("Ctx installed during core_init");
        registration::register_command(self.env, name, id)
    }
}

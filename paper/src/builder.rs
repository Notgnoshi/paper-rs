use jni::Env;
use jni::objects::JObject;

use crate::api::Api;
use crate::bukkit::CommandSenderInst;
use crate::bukkit::event::Event;
use crate::{dispatch, registration};

pub struct PluginBuilder<'a, 'local> {
    pub(crate) env: &'a mut Env<'local>,
    pub(crate) plugin: &'a JObject<'local>,
}

impl<'a, 'local> PluginBuilder<'a, 'local> {
    pub(crate) fn new(env: &'a mut Env<'local>, plugin: &'a JObject<'local>) -> Self {
        Self { env, plugin }
    }

    /// Register a Bukkit event handler.
    ///
    /// The event type is identified by an implementation of [`Event`] (typically a marker type in
    /// `paper::bukkit::event`); the handler receives the corresponding `Event::Wrapper` for the JNI
    /// frame's lifetime.
    pub fn on<E: Event>(
        &mut self,
        handler: impl for<'b, 'l> Fn(&mut Api<'b, 'l>, &E::Wrapper<'l>) + Send + Sync + 'static,
    ) {
        let id = dispatch::next_handler_id();
        dispatch::insert_event_handler(
            id,
            Box::new(move |env, obj| match E::wrap(env, obj) {
                Ok(wrapper) => {
                    let mut api = Api::new(env);
                    handler(&mut api, wrapper);
                }
                Err(e) => {
                    tracing::warn!(
                        "event dispatch type-check failed for {:?}: {e}",
                        E::CLASS_NAME
                    );
                    env.exception_clear();
                }
            }),
        );
        if let Err(e) = registration::subscribe_event(self.env, self.plugin, E::CLASS_NAME, id) {
            tracing::warn!(
                "registering event handler for {:?} failed: {e}",
                E::CLASS_NAME
            );
            self.env.exception_clear();
        }
    }

    /// Register a Bukkit command handler under `name`.
    ///
    /// Returns true from the handler to indicate the command was handled, false to let Bukkit print
    /// usage.
    pub fn command(
        &mut self,
        name: &str,
        handler: impl for<'b, 'l> Fn(&mut Api<'b, 'l>, &CommandSenderInst<'l>, &[String]) -> bool
        + Send
        + Sync
        + 'static,
    ) {
        let id = dispatch::next_handler_id();
        dispatch::insert_command_handler(
            id,
            Box::new(move |env, sender_obj, args| {
                match CommandSenderInst::wrap_ref(env, sender_obj) {
                    Ok(sender) => {
                        let mut api = Api::new(env);
                        handler(&mut api, sender, args)
                    }
                    Err(e) => {
                        tracing::warn!("command dispatch type-check failed: {e}");
                        env.exception_clear();
                        false
                    }
                }
            }),
        );
        if let Err(e) = registration::register_command(self.env, self.plugin, name, id) {
            tracing::warn!("registering command {name:?} failed: {e}");
            self.env.exception_clear();
        }
    }
}

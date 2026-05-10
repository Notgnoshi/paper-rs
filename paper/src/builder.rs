use jni::Env;
use jni::objects::JObject;
use jni::strings::JNIStr;

use crate::api::Api;
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
    /// `event_class` is the JNI class name (e.g.,
    /// `jni_str!("org/bukkit/event/player/PlayerInteractEntityEvent")`).
    pub fn on<F>(&mut self, event_class: &'static JNIStr, handler: F)
    where
        F: for<'b, 'l> Fn(&mut Api<'b, 'l>, &JObject<'l>) + Send + Sync + 'static,
    {
        let id = dispatch::next_handler_id();
        dispatch::insert_event_handler(
            id,
            Box::new(move |env, obj| {
                let mut api = Api::new(env);
                handler(&mut api, obj);
            }),
        );
        if let Err(e) = registration::subscribe_event(self.env, self.plugin, event_class, id) {
            tracing::warn!("registering event handler for {event_class:?} failed: {e}");
            self.env.exception_clear();
        }
    }

    /// Register a Bukkit command handler under `name`.
    ///
    /// Returns true from the handler to indicate the command was handled, false to let Bukkit print
    /// usage.
    pub fn command<F>(&mut self, name: &str, handler: F)
    where
        F: for<'b, 'l> Fn(&mut Api<'b, 'l>, &JObject<'l>, &[String]) -> bool
            + Send
            + Sync
            + 'static,
    {
        let id = dispatch::next_handler_id();
        dispatch::insert_command_handler(
            id,
            Box::new(move |env, sender, args| {
                let mut api = Api::new(env);
                handler(&mut api, sender, args)
            }),
        );
        if let Err(e) = registration::register_command(self.env, self.plugin, name, id) {
            tracing::warn!("registering command {name:?} failed: {e}");
            self.env.exception_clear();
        }
    }
}

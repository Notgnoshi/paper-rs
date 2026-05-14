use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::api::Api;
use crate::bukkit::CommandSenderInst;
use crate::bukkit::event::Event;
use crate::plugin::Plugin;
use crate::{ctx, registration};

/// Plugin setup wrapper around [Api].
///
/// Exposes registration methods (`register_event`, `register_command`, ...) that are only valid
/// during [Plugin::on_enable]. After `on_enable` returns, plugin handlers receive a plain [`Api`]
/// without registration methods.
///
/// Use [SetupApi::api] if a runtime [Api] method is needed during setup (for example, to look up a
/// class).
pub struct SetupApi<'a, 'local, P: Plugin> {
    api: Api<'a, 'local>,
    _phantom: PhantomData<fn() -> P>,
}

impl<'a, 'local, P: Plugin> SetupApi<'a, 'local, P> {
    pub(crate) fn new(api: Api<'a, 'local>) -> Self {
        Self {
            api,
            _phantom: PhantomData,
        }
    }

    /// Borrow the underlying [Api] for runtime methods (class lookup, etc.) during setup.
    pub fn api(&mut self) -> &mut Api<'a, 'local> {
        &mut self.api
    }

    /// Register a Bukkit event handler as a method on the plugin struct.
    ///
    /// The handler runs with `&mut Self` borrowed from the plugin instance papermc is holding for
    /// this `/reload`s lifetime. Plugin state mutated through the handler persists for the rest of
    /// the load (cleared on `/reload` along with the plugin instance).
    pub fn register_event<E: Event>(
        &mut self,
        handler: for<'b, 'l> fn(&mut P, &mut Api<'b, 'l>, &E::Wrapper<'l>),
    ) -> eyre::Result<()> {
        let id = ctx::with_ctx(|c| {
            let id = c.next_handler_id();
            c.event_handlers.insert(
                id,
                Arc::new(move |env, obj| match E::wrap(env, obj) {
                    Ok(wrapper) => {
                        with_plugin::<P, _>(env, |p, env| {
                            let mut api = Api::new(env);
                            handler(p, &mut api, wrapper);
                        });
                    }
                    Err(jni::errors::Error::WrongObjectType) => {
                        tracing::debug!(
                            "event skipped: expected {} (no concrete-type log without builder context)",
                            E::CLASS_NAME,
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "event dispatch type-check failed for {:?}: {e}",
                            E::CLASS_NAME,
                        );
                    }
                }),
            );
            id
        })
        .expect("Ctx installed during plugin_init");
        registration::subscribe_event(self.api.jni(), E::CLASS_NAME, id)?;
        Ok(())
    }

    /// Register a Bukkit command handler as a method on the plugin struct.
    pub fn register_command(
        &mut self,
        name: &str,
        handler: for<'b, 'l> fn(
            &mut P,
            &mut Api<'b, 'l>,
            &CommandSenderInst<'l>,
            &[String],
        ) -> bool,
    ) -> eyre::Result<()> {
        let id = ctx::with_ctx(|c| {
            let id = c.next_handler_id();
            c.command_handlers.insert(
                id,
                Arc::new(move |env, sender_obj, args| {
                    match CommandSenderInst::wrap_ref(env, sender_obj) {
                        Ok(sender) => with_plugin::<P, _>(env, |p, env| {
                            let mut api = Api::new(env);
                            handler(p, &mut api, sender, args)
                        })
                        .unwrap_or(false),
                        Err(e) => {
                            tracing::warn!("command dispatch type-check failed: {e}");
                            false
                        }
                    }
                }),
            );
            id
        })
        .expect("Ctx installed during plugin_init");
        registration::register_command(self.api.jni(), name, id)?;
        Ok(())
    }
}

/// Take the plugin out of `Ctx`, downcast to `P`, run `body(&mut p, env)`, then put the plugin
/// back. Returns `None` if no plugin is currently present.
///
/// Note on reentrancy: the plugin is out of `Ctx` for the duration of `body`. A nested dispatch
/// that re-enters this function would see `None` and silently skip. Bukkit dispatches events on the
/// main server thread, so the more realistic risk is a handler firing another event synchronously
fn with_plugin<'local, P, R>(
    env: &mut jni::Env<'local>,
    body: impl FnOnce(&mut P, &mut jni::Env<'local>) -> R,
) -> Option<R>
where
    P: Plugin,
{
    let mut plugin_box: Box<dyn Any + Send> = ctx::with_ctx(|c| c.rust_plugin.take()).flatten()?;
    let result = {
        let p = plugin_box
            .downcast_mut::<P>()
            .expect("plugin type mismatch in dispatch");
        body(p, env)
    };
    ctx::with_ctx(|c| {
        c.rust_plugin = Some(plugin_box);
    });
    Some(result)
}

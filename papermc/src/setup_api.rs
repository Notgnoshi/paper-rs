use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

use jni::Env;
use jni::objects::JObject;

use crate::api::Api;
use crate::bukkit::CommandSenderInst;
use crate::bukkit::event::Event;
use crate::jobject_repr::JClassCast;
use crate::plugin::Plugin;
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

    /// Register a Bukkit event handler. Handler errors are logged and discarded.
    pub fn register_event<E, F>(&mut self, handler: F) -> eyre::Result<()>
    where
        E: Event,
        F: for<'b, 'l> Fn(&mut P, &mut Api<'b, 'l>, &E::Wrapper<'l>) -> eyre::Result<()>
            + Send
            + Sync
            + 'static,
    {
        let id = ctx::next_id();
        ctx::with_ctx(|c| {
            c.event_handlers.insert(
                id,
                Arc::new(move |env, obj| match E::wrap(env, obj) {
                    Ok(wrapper) => {
                        with_plugin::<P, _>(env, |p, env| {
                            let mut api = Api::new(env);
                            if let Err(e) = handler(p, &mut api, wrapper) {
                                tracing::warn!(
                                    "event handler for {} returned error: {e:?}",
                                    E::CLASS_NAME,
                                );
                            }
                        });
                    }
                    Err(jni::errors::Error::WrongObjectType) => {
                        tracing::debug!(
                            "event skipped: expected {}, actual {}",
                            E::CLASS_NAME,
                            actual_class_name(env, obj),
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
        })
        .expect("Ctx installed during plugin_init");
        registration::subscribe_event(self.api.jni(), E::CLASS_NAME, id)?;
        Ok(())
    }

    /// Register a Bukkit command handler. Handler errors are logged and treated as `false`
    /// (Bukkit will print usage).
    pub fn register_command<F>(&mut self, name: &str, handler: F) -> eyre::Result<()>
    where
        F: for<'b, 'l> Fn(
                &mut P,
                &mut Api<'b, 'l>,
                &CommandSenderInst<'l>,
                &[String],
            ) -> eyre::Result<bool>
            + Send
            + Sync
            + 'static,
    {
        let id = ctx::next_id();
        let command_name = name.to_string();
        ctx::with_ctx(|c| {
            c.command_handlers.insert(
                id,
                Arc::new(move |env, sender_obj, args| {
                    match CommandSenderInst::wrap_ref(env, sender_obj) {
                        Ok(sender) => with_plugin::<P, _>(env, |p, env| {
                            let mut api = Api::new(env);
                            match handler(p, &mut api, sender, args) {
                                Ok(b) => b,
                                Err(e) => {
                                    tracing::warn!(
                                        "command `{command_name}` handler returned error: {e:?}"
                                    );
                                    false
                                }
                            }
                        })
                        .unwrap_or(false),
                        Err(e) => {
                            tracing::warn!("command dispatch type-check failed: {e}");
                            false
                        }
                    }
                }),
            );
        })
        .expect("Ctx installed during plugin_init");
        registration::register_command(self.api.jni(), name, id)?;
        Ok(())
    }
}

/// Take the plugin out of `Ctx`, downcast to `P`, run `body(&mut p, env)`, then put the plugin
/// back. Returns `None` if no plugin is currently present.
///
/// A panic in `body` is caught so the plugin Box is returned to `Ctx` before propagating up to
/// `ffi::bridge`. The plugin's post-panic state may be partially-mutated (which is why we wrap in
/// `AssertUnwindSafe`); preserving it is preferable to silently disabling the plugin until reload.
fn with_plugin<'local, P, R>(
    env: &mut jni::Env<'local>,
    body: impl FnOnce(&mut P, &mut jni::Env<'local>) -> R,
) -> Option<R>
where
    P: Plugin,
{
    let mut plugin_box: Box<dyn Any + Send> = ctx::with_ctx(|c| c.rust_plugin.take()).flatten()?;
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let p = plugin_box
            .downcast_mut::<P>()
            .expect("plugin type mismatch in dispatch");
        body(p, env)
    }));
    ctx::with_ctx(|c| {
        c.rust_plugin = Some(plugin_box);
    });
    match outcome {
        Ok(r) => Some(r),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

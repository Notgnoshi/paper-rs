use jni::Env;
use jni::objects::JClass;

use crate::ctx::{self, Ctx};

pub struct Api<'a, 'local> {
    env: &'a mut Env<'local>,
}

impl<'a, 'local> Api<'a, 'local> {
    pub(crate) fn new(env: &'a mut Env<'local>) -> Self {
        Self { env }
    }

    /// Provide raw `jni::Env` access for circumstances where [Api]s API is insufficient.
    pub fn jni(&mut self) -> &mut Env<'local> {
        self.env
    }

    /// Resolve a JNI class by name, caching the global so subsequent lookups skip `FindClass`.
    ///
    /// `name` is the slash-delimited JVM form (e.g. `"org/bukkit/entity/Player"`). The returned
    /// `JClass<'local>` is a fresh local ref tied to the current JNI frame; the global stays in
    /// the per-plugin cache for the lifetime of the load.
    pub fn class(&mut self, name: &'static str) -> eyre::Result<JClass<'local>> {
        Ok(ctx::cached_class(self.env, name)?)
    }

    /// Run `body` with mutable access to the per-plugin [`Ctx`].
    ///
    /// Returns `None` if Ctx is not initialized (shouldn't happen from a handler; Ctx is alive
    /// between `core_init` and `core_shutdown`).
    pub(crate) fn with_ctx<R>(&mut self, body: impl FnOnce(&mut Ctx) -> R) -> Option<R> {
        ctx::with_ctx(body)
    }
}

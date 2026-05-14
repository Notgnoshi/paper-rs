use jni::Env;
use jni::objects::JClass;

use crate::ctx;

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
}

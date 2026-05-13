use jni::Env;

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

    /// Run `body` with mutable access to the per-plugin [`Ctx`].
    ///
    /// Returns `None` if Ctx is not initialized (shouldn't happen from a handler; Ctx is alive
    /// between `core_init` and `core_shutdown`).
    pub(crate) fn with_ctx<R>(&mut self, body: impl FnOnce(&mut Ctx) -> R) -> Option<R> {
        ctx::with_ctx(body)
    }
}

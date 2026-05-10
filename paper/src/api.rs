use jni::Env;

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
}

use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;

/// Wrapper for a `net.kyori.adventure.key.Key` JNI reference.
#[repr(transparent)]
pub struct Key<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> Key<'local> {
    /// Construct a Key from a namespace and value, e.g. `Key::key(api, "disco", "sheep_baaa")`
    /// for `disco:sheep_baaa`.
    pub fn key(
        api: &mut Api<'_, 'local>,
        namespace: &str,
        value: &str,
    ) -> jni::errors::Result<Self> {
        let env = api.jni();
        let ns = env.new_string(namespace)?;
        let val = env.new_string(value)?;
        let obj = env
            .call_static_method(
                jni_str!("net/kyori/adventure/key/Key"),
                jni_str!("key"),
                jni_sig!("(Ljava/lang/String;Ljava/lang/String;)Lnet/kyori/adventure/key/Key;"),
                &[JValue::Object(&ns), JValue::Object(&val)],
            )?
            .l()?;
        Ok(Self { obj })
    }
}

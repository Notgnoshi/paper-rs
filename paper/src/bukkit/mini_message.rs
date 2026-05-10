use std::sync::Mutex;

use jni::objects::{JObject, JValue};
use jni::refs::Global;
use jni::{Env, jni_sig, jni_str};

/// Cached `MiniMessage` singleton.
///
/// Lazy-initialized on first use; cleared on `core_shutdown` so the JVM-side ref drops before
/// disco-core's .so unloads.
static MINI_MESSAGE: Mutex<Option<Global<JObject<'static>>>> = Mutex::new(None);

/// Get (or lazily fetch) a local reference to the MiniMessage singleton for this JNI frame.
fn instance<'local>(env: &mut Env<'local>) -> jni::errors::Result<JObject<'local>> {
    let mut guard = MINI_MESSAGE.lock().unwrap();
    if guard.is_none() {
        let inst = env
            .call_static_method(
                jni_str!("net/kyori/adventure/text/minimessage/MiniMessage"),
                jni_str!("miniMessage"),
                jni_sig!("()Lnet/kyori/adventure/text/minimessage/MiniMessage;"),
                &[],
            )?
            .l()?;
        *guard = Some(env.new_global_ref(&inst)?);
    }
    let global = guard.as_ref().unwrap();
    env.new_local_ref(global)
}

/// Parse `text` as MiniMessage and return the resulting Adventure `Component` JNI ref.
///
/// Plain text without tags goes through unchanged.
pub(crate) fn deserialize<'local>(
    env: &mut Env<'local>,
    text: &str,
) -> jni::errors::Result<JObject<'local>> {
    let inst = instance(env)?;
    let jstr = env.new_string(text)?;
    env.call_method(
        &inst,
        jni_str!("deserialize"),
        jni_sig!("(Ljava/lang/String;)Lnet/kyori/adventure/text/Component;"),
        &[JValue::Object(&jstr)],
    )?
    .l()
}

/// Drop the cached MiniMessage global ref.
///
/// Called from `core_shutdown` so the JNI ref is released before disco-core's .so unloads.
pub(crate) fn shutdown() {
    *MINI_MESSAGE.lock().unwrap() = None;
}

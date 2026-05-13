use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};

use crate::ctx;

/// Get (or lazily fetch) a local reference to the MiniMessage singleton for this JNI frame.
fn instance<'local>(env: &mut Env<'local>) -> jni::errors::Result<JObject<'local>> {
    ctx::with_ctx(|c| -> jni::errors::Result<JObject<'local>> {
        if c.mini_message.is_none() {
            let inst = env
                .call_static_method(
                    jni_str!("net/kyori/adventure/text/minimessage/MiniMessage"),
                    jni_str!("miniMessage"),
                    jni_sig!("()Lnet/kyori/adventure/text/minimessage/MiniMessage;"),
                    &[],
                )?
                .l()?;
            c.mini_message = Some(env.new_global_ref(&inst)?);
        }
        let global = c.mini_message.as_ref().unwrap();
        env.new_local_ref(global)
    })
    .expect("Ctx installed during core_init")
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

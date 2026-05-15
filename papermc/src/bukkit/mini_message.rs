use std::sync::Arc;

use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};

use crate::ctx;

fn instance<'local>(env: &mut Env<'local>) -> jni::errors::Result<JObject<'local>> {
    let cached =
        ctx::with_ctx(|c| c.mini_message.clone()).expect("Ctx installed during plugin_init");
    let global = match cached {
        Some(g) => g,
        None => {
            let inst = env
                .call_static_method(
                    jni_str!("net/kyori/adventure/text/minimessage/MiniMessage"),
                    jni_str!("miniMessage"),
                    jni_sig!("()Lnet/kyori/adventure/text/minimessage/MiniMessage;"),
                    &[],
                )?
                .l()?;
            let new_global = Arc::new(env.new_global_ref(&inst)?);
            ctx::with_ctx(|c| {
                c.mini_message
                    .get_or_insert_with(|| new_global.clone())
                    .clone()
            })
            .expect("Ctx installed during plugin_init")
        }
    };
    env.new_local_ref(&*global)
}

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

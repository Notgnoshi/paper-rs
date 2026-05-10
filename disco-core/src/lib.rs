use jni::objects::{JObject, JString, JValue};
use jni::sys::{JNIEnv, jobject};
use jni::{jni_sig, jni_str};
use paper::{Api, CoreApi, PluginBuilder};

#[unsafe(no_mangle)]
pub extern "C" fn paper_core_init(env: *mut JNIEnv, plugin: jobject) -> *const CoreApi {
    paper::core_init(env, plugin, |b: &mut PluginBuilder| {
        b.on(
            jni_str!("org/bukkit/event/player/PlayerInteractEntityEvent"),
            handle_interact,
        );
        b.command("hello", handle_hello);
    })
}

fn handle_interact(api: &mut Api, event: &JObject<'_>) {
    let env = api.jni();
    let result = (|| -> jni::errors::Result<()> {
        let entity = env
            .call_method(
                event,
                jni_str!("getRightClicked"),
                jni_sig!("()Lorg/bukkit/entity/Entity;"),
                &[],
            )?
            .l()?;
        let sheep_class = env.find_class(jni_str!("org/bukkit/entity/Sheep"))?;
        if !env.is_instance_of(&entity, &sheep_class)? {
            return Ok(());
        }
        tracing::debug!("Player interacted with a sheep, changing its color to pink: {entity:?}");
        let pink = env
            .get_static_field(
                jni_str!("org/bukkit/DyeColor"),
                jni_str!("PINK"),
                jni_sig!("Lorg/bukkit/DyeColor;"),
            )?
            .l()?;
        env.call_method(
            &entity,
            jni_str!("setColor"),
            jni_sig!("(Lorg/bukkit/DyeColor;)V"),
            &[JValue::Object(&pink)],
        )?;
        Ok(())
    })();
    if let Err(e) = result {
        tracing::warn!("interact handler failed: {e}");
        let _ = env.exception_clear();
    }
}

/// Build the greeting reply for the /hello command.
pub fn hello(name: &str) -> String {
    tracing::debug!("Greeting {name}");
    format!("Hello, {name}!")
}

fn handle_hello(api: &mut Api, sender: &JObject<'_>, args: &[String]) -> bool {
    let env = api.jni();
    let result = (|| -> jni::errors::Result<()> {
        let name = if let Some(arg) = args.first() {
            arg.clone()
        } else {
            let name_obj = env
                .call_method(
                    sender,
                    jni_str!("getName"),
                    jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?;
            let name_jstr = env.cast_local::<JString>(name_obj)?;
            name_jstr.try_to_string(env)?
        };
        let reply = hello(&name);
        let reply_jstr = env.new_string(&reply)?;
        env.call_method(
            sender,
            jni_str!("sendMessage"),
            jni_sig!("(Ljava/lang/String;)V"),
            &[JValue::Object(&reply_jstr)],
        )?;
        Ok(())
    })();
    if let Err(e) = result {
        tracing::warn!("hello handler failed: {e}");
        let _ = env.exception_clear();
        return false;
    }
    true
}

use jni::objects::{JObject, JValue};
use jni::{Env, jni_sig, jni_str};

use crate::ctx;

pub(crate) fn subscribe_event<'local>(
    env: &mut Env<'local>,
    event_class_name: &'static str,
    handler_id: i64,
) -> jni::errors::Result<()> {
    let event_class = ctx::cached_class(env, event_class_name)?;
    let executor = env.new_object(
        jni_str!("io/papermc/RustEventExecutor"),
        jni_sig!("(J)V"),
        &[JValue::Long(handler_id)],
    )?;
    let priority = env
        .get_static_field(
            jni_str!("org/bukkit/event/EventPriority"),
            jni_str!("NORMAL"),
            jni_sig!("Lorg/bukkit/event/EventPriority;"),
        )?
        .l()?;
    let plugin =
        ctx::with_ctx(|c| c.java_plugin.clone()).expect("Ctx installed during plugin_init");
    let server = env
        .call_method(
            &*plugin,
            jni_str!("getServer"),
            jni_sig!("()Lorg/bukkit/Server;"),
            &[],
        )?
        .l()?;
    let plugin_manager = env
        .call_method(
            &server,
            jni_str!("getPluginManager"),
            jni_sig!("()Lorg/bukkit/plugin/PluginManager;"),
            &[],
        )?
        .l()?;
    let event_class_obj = JObject::from(event_class);
    env.call_method(
        &plugin_manager,
        jni_str!("registerEvent"),
        jni_sig!(
            "(Ljava/lang/Class;Lorg/bukkit/event/Listener;Lorg/bukkit/event/EventPriority;Lorg/bukkit/plugin/EventExecutor;Lorg/bukkit/plugin/Plugin;)V"
        ),
        &[
            JValue::Object(&event_class_obj),
            JValue::Object(&executor),
            JValue::Object(&priority),
            JValue::Object(&executor),
            JValue::Object(&*plugin),
        ],
    )?;
    Ok(())
}

pub(crate) fn register_command<'local>(
    env: &mut Env<'local>,
    name: &str,
    handler_id: i64,
) -> jni::errors::Result<()> {
    let name_jstr = env.new_string(name)?;
    let command = env.new_object(
        jni_str!("io/papermc/RustCommand"),
        jni_sig!("(Ljava/lang/String;J)V"),
        &[JValue::Object(&name_jstr), JValue::Long(handler_id)],
    )?;
    let fallback = env.new_string("papermc")?;
    let plugin =
        ctx::with_ctx(|c| c.java_plugin.clone()).expect("Ctx installed during plugin_init");
    let server = env
        .call_method(
            &*plugin,
            jni_str!("getServer"),
            jni_sig!("()Lorg/bukkit/Server;"),
            &[],
        )?
        .l()?;
    let command_map = env
        .call_method(
            &server,
            jni_str!("getCommandMap"),
            jni_sig!("()Lorg/bukkit/command/CommandMap;"),
            &[],
        )?
        .l()?;
    env.call_method(
        &command_map,
        jni_str!("register"),
        jni_sig!("(Ljava/lang/String;Lorg/bukkit/command/Command;)Z"),
        &[JValue::Object(&fallback), JValue::Object(&command)],
    )?;
    let cmd_global = env.new_global_ref(&command)?;
    ctx::with_ctx(|c| c.registered_commands.push(cmd_global))
        .expect("Ctx installed during plugin_init");
    Ok(())
}

pub(crate) fn unregister_commands(env: &mut Env<'_>) -> jni::errors::Result<()> {
    let commands =
        ctx::with_ctx(|c| std::mem::take(&mut c.registered_commands)).unwrap_or_default();
    if commands.is_empty() {
        return Ok(());
    }
    let server = env
        .call_static_method(
            jni_str!("org/bukkit/Bukkit"),
            jni_str!("getServer"),
            jni_sig!("()Lorg/bukkit/Server;"),
            &[],
        )?
        .l()?;
    let command_map = env
        .call_method(
            &server,
            jni_str!("getCommandMap"),
            jni_sig!("()Lorg/bukkit/command/CommandMap;"),
            &[],
        )?
        .l()?;
    for cmd in commands {
        let _ = env.call_method(
            &cmd,
            jni_str!("unregister"),
            jni_sig!("(Lorg/bukkit/command/CommandMap;)Z"),
            &[JValue::Object(&command_map)],
        );
    }
    Ok(())
}

/// Must run before handler-map teardown; otherwise an event in flight between teardown and
/// Bukkit's own listener cleanup logs a spurious "no handler registered" warning.
pub(crate) fn unregister_all_listeners(env: &mut Env<'_>) -> jni::errors::Result<()> {
    let plugin =
        ctx::with_ctx(|c| c.java_plugin.clone()).expect("Ctx installed during plugin_init");
    env.call_static_method(
        jni_str!("org/bukkit/event/HandlerList"),
        jni_str!("unregisterAll"),
        jni_sig!("(Lorg/bukkit/plugin/Plugin;)V"),
        &[JValue::Object(&*plugin)],
    )?;
    Ok(())
}

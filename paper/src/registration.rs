use std::sync::Mutex;

use jni::objects::{JObject, JValue};
use jni::refs::Global;
use jni::strings::JNIStr;
use jni::{Env, jni_sig, jni_str};

/// Tracks `RustCommand` instances we registered with Bukkit's CommandMap so we can `unregister`
/// them on shutdown.
///
/// Otherwise stale instances accumulate across /reload cycles, each pointing at a defunct
/// handlerId.
static REGISTERED_COMMANDS: Mutex<Vec<Global<JObject<'static>>>> = Mutex::new(Vec::new());

/// Construct a `RustEventExecutor(handlerId)` and register it with the PluginManager for the given
/// event class.
pub(crate) fn subscribe_event<'local>(
    env: &mut Env<'local>,
    plugin: &JObject<'local>,
    event_class_name: &'static JNIStr,
    handler_id: i64,
) -> jni::errors::Result<()> {
    let event_class = env.find_class(event_class_name)?;
    let executor = env.new_object(
        jni_str!("io/paperrs/shim/RustEventExecutor"),
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
    let server = env
        .call_method(
            plugin,
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
            JValue::Object(plugin),
        ],
    )?;
    Ok(())
}

/// Construct a `RustCommand(name, handlerId)` and register it with the server's CommandMap.
///
/// The instance is tracked in `REGISTERED_COMMANDS` for cleanup on shutdown.
pub(crate) fn register_command<'local>(
    env: &mut Env<'local>,
    plugin: &JObject<'local>,
    name: &str,
    handler_id: i64,
) -> jni::errors::Result<()> {
    let name_jstr = env.new_string(name)?;
    let command = env.new_object(
        jni_str!("io/paperrs/shim/RustCommand"),
        jni_sig!("(Ljava/lang/String;J)V"),
        &[JValue::Object(&name_jstr), JValue::Long(handler_id)],
    )?;
    let server = env
        .call_method(
            plugin,
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
    let fallback = env.new_string("paper-rs")?;
    env.call_method(
        &command_map,
        jni_str!("register"),
        jni_sig!("(Ljava/lang/String;Lorg/bukkit/command/Command;)Z"),
        &[JValue::Object(&fallback), JValue::Object(&command)],
    )?;
    let cmd_global = env.new_global_ref(&command)?;
    REGISTERED_COMMANDS.lock().unwrap().push(cmd_global);
    Ok(())
}

/// Walk the tracked `RustCommand` instances and call `Command.unregister(commandMap)` on each.
///
/// Called from `core_shutdown`.
pub(crate) fn unregister_commands(env: &mut Env<'_>) -> jni::errors::Result<()> {
    let commands = std::mem::take(&mut *REGISTERED_COMMANDS.lock().unwrap());
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
        // cmd's Drop calls DeleteGlobalRef when this scope ends.
    }
    Ok(())
}

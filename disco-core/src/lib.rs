//! `disco-core` cdylib: the per-plugin Rust code dlopen'd by paper-loader.

use std::collections::HashMap;
use std::mem::size_of;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};

use jni::objects::{JObject, JObjectArray, JString, JValue};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jlong, jobject, jobjectArray};
use jni::{Env, EnvUnowned, jni_sig, jni_str};
use paper::{CORE_ABI_VERSION, CoreApi};
use tracing::{info, warn};

// ---- Pure-Rust logic --------------------------------------------------------

/// Build the greeting reply for the /hello command.
pub fn hello(name: &str) -> String {
    tracing::debug!("Greeting {name}");
    format!("Hello, {name}!")
}

// ---- Handler registries -----------------------------------------------------

type EventHandler = Box<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>) + Send + Sync>;
type CommandHandler =
    Box<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>, &[String]) -> bool + Send + Sync>;

static EVENT_HANDLERS: OnceLock<Mutex<HashMap<i64, EventHandler>>> = OnceLock::new();
static COMMAND_HANDLERS: OnceLock<Mutex<HashMap<i64, CommandHandler>>> = OnceLock::new();
static NEXT_HANDLER_ID: AtomicI64 = AtomicI64::new(1);

fn event_handlers() -> &'static Mutex<HashMap<i64, EventHandler>> {
    EVENT_HANDLERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn command_handlers() -> &'static Mutex<HashMap<i64, CommandHandler>> {
    COMMAND_HANDLERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_handler_id() -> i64 {
    NEXT_HANDLER_ID.fetch_add(1, Ordering::SeqCst)
}

// ---- Bukkit registration helpers --------------------------------------------

/// Register an event handler with Bukkit by:
///   1. allocating a Rust-side handler id,
///   2. constructing a `RustEventExecutor(handlerId)`,
///   3. calling plugin.getServer().getPluginManager().registerEvent(...).
fn subscribe_event<'local>(
    env: &mut Env<'local>,
    plugin: &JObject<'local>,
    event_class_name: &'static jni::strings::JNIStr,
    handler: EventHandler,
) -> jni::errors::Result<()> {
    let handler_id = next_handler_id();
    event_handlers().lock().unwrap().insert(handler_id, handler);

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

/// Register a command handler with Bukkit by constructing a `RustCommand` and
/// adding it to the server's `CommandMap`.
fn register_command<'local>(
    env: &mut Env<'local>,
    plugin: &JObject<'local>,
    name: &str,
    handler: CommandHandler,
) -> jni::errors::Result<()> {
    let handler_id = next_handler_id();
    command_handlers()
        .lock()
        .unwrap()
        .insert(handler_id, handler);

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
    let fallback = env.new_string("disco")?;
    env.call_method(
        &command_map,
        jni_str!("register"),
        jni_sig!("(Ljava/lang/String;Lorg/bukkit/command/Command;)Z"),
        &[JValue::Object(&fallback), JValue::Object(&command)],
    )?;
    Ok(())
}

// ---- Spike handlers (will move out in stage 4-5) ---------------------------

fn handle_player_interact_entity<'local>(env: &mut Env<'local>, event: &JObject<'local>) {
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
        warn!("interact handler failed: {e}");
        let _ = env.exception_clear();
    }
}

fn handle_hello<'local>(env: &mut Env<'local>, sender: &JObject<'local>, args: &[String]) -> bool {
    let result = (|| -> jni::errors::Result<()> {
        let name = if let Some(arg) = args.first() {
            arg.clone()
        } else {
            // sender.getName()
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
        warn!("hello handler failed: {e}");
        let _ = env.exception_clear();
        return false;
    }
    true
}

// ---- CoreApi function pointers ---------------------------------------------

unsafe extern "C" fn core_init(env: *mut jni::sys::JNIEnv, plugin: jobject) -> i32 {
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let outcome = unowned.with_env(|env: &mut Env<'_>| -> jni::errors::Result<()> {
        paper::install_logger(env)?;
        info!("paper_core_init: installing handlers");
        let plugin_obj = unsafe { JObject::from_raw(env, plugin) };
        subscribe_event(
            env,
            &plugin_obj,
            jni_str!("org/bukkit/event/player/PlayerInteractEntityEvent"),
            Box::new(handle_player_interact_entity),
        )?;
        register_command(env, &plugin_obj, "hello", Box::new(handle_hello))?;
        Ok(())
    });
    match outcome.into_outcome() {
        jni::Outcome::Ok(_) => 0,
        jni::Outcome::Err(e) => {
            warn!("core_init failed: {e}");
            1
        }
        jni::Outcome::Panic(_) => {
            warn!("core_init panicked");
            2
        }
    }
}

unsafe extern "C" fn core_shutdown(_env: *mut jni::sys::JNIEnv) -> i32 {
    // Stage 3 implements proper cleanup; stage 2 just logs.
    info!("paper_core_shutdown (stage 2 stub)");
    0
}

unsafe extern "C" fn core_dispatch_event(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    event: jobject,
) {
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let _ = unowned
        .with_env(|env: &mut Env<'_>| -> jni::errors::Result<()> {
            let map = event_handlers().lock().unwrap();
            let Some(handler) = map.get(&handler_id) else {
                warn!("no event handler registered for id {handler_id}");
                return Ok(());
            };
            let event_obj = unsafe { JObject::from_raw(env, event) };
            handler(env, &event_obj);
            Ok(())
        })
        .into_outcome();
}

unsafe extern "C" fn core_dispatch_command(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jboolean {
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let outcome = unowned.with_env(|env: &mut Env<'_>| -> jni::errors::Result<bool> {
        let map = command_handlers().lock().unwrap();
        let Some(handler) = map.get(&handler_id) else {
            warn!("no command handler registered for id {handler_id}");
            return Ok(false);
        };
        let sender_obj = unsafe { JObject::from_raw(env, sender) };
        let args_arr = unsafe { JObjectArray::<JString>::from_raw(env, args) };
        let args_vec = read_string_array(env, &args_arr)?;
        let result = handler(env, &sender_obj, &args_vec);
        Ok(result)
    });
    match outcome.into_outcome() {
        jni::Outcome::Ok(b) => {
            if b {
                JNI_TRUE
            } else {
                JNI_FALSE
            }
        }
        _ => JNI_FALSE,
    }
}

unsafe extern "C" fn core_dispatch_tab_complete(
    _env: *mut jni::sys::JNIEnv,
    _handler_id: jlong,
    _sender: jobject,
    _args: jobjectArray,
) -> jobject {
    std::ptr::null_mut()
}

fn read_string_array(
    env: &mut Env<'_>,
    arr: &JObjectArray<'_, JString>,
) -> jni::errors::Result<Vec<String>> {
    let len = arr.len(env)?;
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let elem = arr.get_element(env, i)?;
        let s = elem.try_to_string(env)?;
        out.push(s);
    }
    Ok(out)
}

// ---- The single C-ABI export the loader looks for --------------------------

/// Returns the static `CoreApi` table for this core.
///
/// Loader calls this once after dlopen, validates the version + size, then
/// dispatches all subsequent JNI calls through the function pointers in the
/// returned struct.
#[unsafe(no_mangle)]
pub extern "C" fn paper_core_init() -> *const CoreApi {
    static CORE_API: CoreApi = CoreApi {
        abi_version: CORE_ABI_VERSION,
        size: size_of::<CoreApi>() as u32,
        init: core_init,
        shutdown: core_shutdown,
        dispatch_event: core_dispatch_event,
        dispatch_command: core_dispatch_command,
        dispatch_tab_complete: core_dispatch_tab_complete,
    };
    &CORE_API
}

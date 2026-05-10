//! `disco-core` cdylib: combined pure-Rust logic + JNI surface for the disco PoC.
//!
//! Stage 1 of the loader-shim migration: this is the merged disco-ffi + old
//! disco-core. Java still loads this .so directly via `System.load` and links to
//! the `Java_io_disco_plugin_*` JNI symbols. Stage 2 replaces this surface with
//! `paper-loader.so` + a `paper_core_init` entry point.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};

use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::jlong;
use jni::{Env, EnvUnowned, jni_sig, jni_str};
use tracing::{info, warn};

// ---- Pure-Rust logic (formerly disco-core) ---------------------------------

/// Build the greeting reply for the /hello command.
pub fn hello(name: &str) -> String {
    tracing::debug!("Greeting {name}");
    format!("Hello, {name}!")
}

/// Pick a DyeColor index (0..=15) for a sheep, varying per-click and per-sheep so
/// rapid clicks cycle through colors.
pub fn pick_sheep_color(uuid: [u8; 16]) -> u8 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let uuid_sum: u32 = uuid.iter().map(|&b| b as u32).sum();
    ((nanos ^ uuid_sum) % 16) as u8
}

// ---- JNI surface (formerly disco-ffi) --------------------------------------

/// JNI native method: HelloCommand.hello(String name) -> String
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_disco_plugin_HelloCommand_hello<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    name: JString<'local>,
) -> JString<'local> {
    unowned_env
        .with_env(
            |env: &mut Env<'local>| -> jni::errors::Result<JString<'local>> {
                let name_str = name.try_to_string(env)?;
                let reply = hello(&name_str);
                env.new_string(&reply)
            },
        )
        .resolve::<ThrowRuntimeExAndDefault>()
}

// ---- Generic event dispatch -------------------------------------------------

type Handler = Box<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>) + Send + Sync>;

static HANDLERS: OnceLock<Mutex<HashMap<i64, Handler>>> = OnceLock::new();
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

fn handlers() -> &'static Mutex<HashMap<i64, Handler>> {
    HANDLERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_handler(handler: Handler) -> i64 {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    handlers().lock().unwrap().insert(id, handler);
    id
}

/// Register an event handler with Bukkit by:
///   1. allocating a Rust-side handler id,
///   2. constructing a `RustEventExecutor(handlerId)` (implements both Listener and
///      EventExecutor),
///   3. calling plugin.getServer().getPluginManager().registerEvent(...).
fn subscribe<'local>(
    env: &mut Env<'local>,
    plugin: &JObject<'local>,
    event_class_name: &'static jni::strings::JNIStr,
    handler: Handler,
) -> jni::errors::Result<()> {
    let handler_id = register_handler(handler);

    let event_class = env.find_class(event_class_name)?;
    let executor = env.new_object(
        jni_str!("io/disco/plugin/RustEventExecutor"),
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

/// Spike handler: on PlayerInteractEntityEvent against a Sheep, set color to PINK.
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

/// JNI native method: DiscoPlugin.discoStart(Plugin plugin)
///
/// Called once at plugin enable time. Installs the JNI logger bridge and
/// registers all Rust-side event handlers.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_disco_plugin_DiscoPlugin_discoStart<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    plugin: JObject<'local>,
) {
    let _ = unowned_env
        .with_env(|env: &mut Env<'local>| -> jni::errors::Result<()> {
            paper::install_logger(env)?;
            info!("discoStart called");
            if let Err(e) = subscribe(
                env,
                &plugin,
                jni_str!("org/bukkit/event/player/PlayerInteractEntityEvent"),
                Box::new(handle_player_interact_entity),
            ) {
                warn!("failed to register interact handler: {e}");
                let _ = env.exception_clear();
            }
            Ok(())
        })
        .into_outcome();
}

/// JNI native method: RustEventExecutor.dispatch(long handlerId, Object event)
///
/// Looks up the Rust handler by id and invokes it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_disco_plugin_RustEventExecutor_dispatch<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handler_id: jlong,
    event: JObject<'local>,
) {
    let _ = unowned_env
        .with_env(|env: &mut Env<'local>| -> jni::errors::Result<()> {
            let map = handlers().lock().unwrap();
            let Some(handler) = map.get(&handler_id) else {
                warn!("no handler registered for id {handler_id}");
                return Ok(());
            };
            handler(env, &event);
            Ok(())
        })
        .into_outcome();
}

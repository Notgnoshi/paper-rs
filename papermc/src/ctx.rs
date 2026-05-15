use std::any::Any;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use jni::Env;
use jni::objects::{JClass, JObject};
use jni::refs::Global;
use jni::strings::JNIStr;

use crate::api::Api;
use crate::callbacks::BiConsumerFn;
use crate::dispatch::{CommandHandler, EventHandler};

pub(crate) type OnDisableFn =
    Box<dyn for<'a, 'local> Fn(&mut dyn Any, &mut Api<'a, 'local>) -> eyre::Result<()> + Send>;

/// Reload-scoped state. Born in `plugin_init`, dropped in `plugin_on_disable`.
pub(crate) struct Ctx {
    pub(crate) java_plugin: Arc<Global<JObject<'static>>>,
    pub(crate) registered_commands: Vec<Global<JObject<'static>>>,
    pub(crate) event_handlers: HashMap<i64, EventHandler>,
    pub(crate) command_handlers: HashMap<i64, CommandHandler>,
    pub(crate) callbacks: HashMap<i64, BiConsumerFn>,
    pub(crate) mini_message: Option<Arc<Global<JObject<'static>>>>,
    jni_cache: HashMap<&'static str, Arc<Global<JClass<'static>>>>,
    pub(crate) rust_plugin: Option<Box<dyn Any + Send>>,
    pub(crate) on_disable_fn: Option<OnDisableFn>,
}

impl Ctx {
    pub(crate) fn new(java_plugin: Global<JObject<'static>>) -> Self {
        Self {
            java_plugin: Arc::new(java_plugin),
            registered_commands: Vec::new(),
            event_handlers: HashMap::new(),
            command_handlers: HashMap::new(),
            callbacks: HashMap::new(),
            mini_message: None,
            jni_cache: HashMap::new(),
            rust_plugin: None,
            on_disable_fn: None,
        }
    }
}

// Outside Ctx so ids don't reset on /reload: stale Cleaner-driven `drop_callback(id)` from the
// prior load would otherwise evict a live callback issued the same id by the new load.
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

pub(crate) fn next_id() -> i64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

static CTX: Mutex<Option<Ctx>> = Mutex::new(None);

pub(crate) struct AlreadyInitialized;

/// Ignore poisoning: a panic mid-mutation leaves Ctx in an unusual but not catastrophic state
/// (each field's operations are simple inserts/takes). Bailing out would brick the plugin until
/// the server restarts.
fn lock() -> std::sync::MutexGuard<'static, Option<Ctx>> {
    CTX.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn install(ctx: Ctx) -> Result<(), AlreadyInitialized> {
    let mut guard = lock();
    if guard.is_some() {
        return Err(AlreadyInitialized);
    }
    *guard = Some(ctx);
    Ok(())
}

pub(crate) fn uninstall() {
    *lock() = None;
}

/// The lock is held for the duration of `body`; `body` must not do JNI or invoke user code.
pub(crate) fn with_ctx<R>(body: impl FnOnce(&mut Ctx) -> R) -> Option<R> {
    let mut guard = lock();
    guard.as_mut().map(body)
}

/// Names must be valid JVM class descriptors (e.g. `org/bukkit/entity/Player`); invalid input
/// panics since every call site passes a compile-time literal.
pub(crate) fn cached_class<'local>(
    env: &mut Env<'local>,
    name: &'static str,
) -> jni::errors::Result<JClass<'local>> {
    let cached =
        with_ctx(|c| c.jni_cache.get(name).cloned()).expect("Ctx installed during plugin_init");
    let global = match cached {
        Some(g) => g,
        None => {
            let cstring =
                CString::new(name).expect("class-name literal contains interior NUL byte");
            let jni_str = JNIStr::from_cstr(&cstring)
                .expect("class-name literal is not valid modified UTF-8");
            let class_local = env.find_class(jni_str)?;
            let class_global = Arc::new(env.new_global_ref(&class_local)?);
            with_ctx(|c| {
                c.jni_cache
                    .entry(name)
                    .or_insert_with(|| class_global.clone())
                    .clone()
            })
            .expect("Ctx installed during plugin_init")
        }
    };
    env.new_local_ref(&*global)
}

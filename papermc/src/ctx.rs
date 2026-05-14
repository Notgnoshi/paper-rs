use std::any::Any;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Mutex;

use jni::Env;
use jni::objects::{JClass, JObject};
use jni::refs::Global;
use jni::strings::JNIStr;

use crate::api::Api;
use crate::callbacks::BiConsumerFn;
use crate::dispatch::{CommandHandler, EventHandler};

/// Type-erased on_disable Fn; downcasts the boxed plugin to its concrete `P` and calls
/// `P::on_disable`. Captured at `init::<P>` time so `plugin_on_disable` can invoke it without knowing
/// `P`.
pub(crate) type OnDisableFn =
    Box<dyn for<'a, 'local> Fn(&mut dyn Any, &mut Api<'a, 'local>) -> eyre::Result<()> + Send>;

/// Reload-scoped state for a Paper plugin.
///
/// [`Ctx`] is the single consolidated home for state that needs to outlive an individual JNI
/// dispatch call but not survive a `/reload`. Born in `plugin_init`, dropped in `plugin_on_disable`.
///
/// The Ctx is stored in a global static, but its lifetime is scoped by the plugin initialization
/// and shutdown.
///
/// User plugin code does not see `Ctx` directly; access is through `crate::Api` helpers.
pub(crate) struct Ctx {
    /// JNI global reference to the Java plugin object. Used wherever a registration site needs
    /// to pass the plugin into a Bukkit call (event subscription, command registration, listener
    /// unregistration).
    pub(crate) java_plugin: Global<JObject<'static>>,
    /// Bukkit `Command` instances we've registered with the CommandMap. Drained at shutdown so
    /// the CommandMap doesn't retain stale handlers across `/reload`.
    pub(crate) registered_commands: Vec<Global<JObject<'static>>>,
    pub(crate) event_handlers: HashMap<i64, EventHandler>,
    pub(crate) command_handlers: HashMap<i64, CommandHandler>,
    pub(crate) callbacks: HashMap<i64, BiConsumerFn>,
    /// Cached `MiniMessage` singleton. Lazy-initialized on first use, since not every plugin
    /// touches MiniMessage.
    pub(crate) mini_message: Option<Global<JObject<'static>>>,
    /// Cache of resolved JNI class globals, keyed by the same `&'static str` JNI class name
    /// that drove the lookup. Populated lazily on first miss in `cached_class`. Cleared along
    /// with the rest of `Ctx` on shutdown, releasing each `DeleteGlobalRef`.
    jni_cache: HashMap<&'static str, Global<JClass<'static>>>,
    /// User plugin instance returned by `Plugin::on_enable`. Held as `Box<dyn Any + Send>` so
    /// papermc doesn't need a generic parameter for the plugin type
    pub(crate) rust_plugin: Option<Box<dyn Any + Send>>,
    pub(crate) on_disable_fn: Option<OnDisableFn>,
    next_handler_id: i64,
    next_callback_id: i64,
}

impl Ctx {
    pub(crate) fn new(java_plugin: Global<JObject<'static>>) -> Self {
        Self {
            java_plugin,
            registered_commands: Vec::new(),
            event_handlers: HashMap::new(),
            command_handlers: HashMap::new(),
            callbacks: HashMap::new(),
            mini_message: None,
            jni_cache: HashMap::new(),
            rust_plugin: None,
            on_disable_fn: None,
            next_handler_id: 1,
            next_callback_id: 1,
        }
    }

    /// Allocate a fresh handler id.
    ///
    /// Ids are unique within a single plugin load; reset to 1 on each plugin reload.
    pub(crate) fn next_handler_id(&mut self) -> i64 {
        let id = self.next_handler_id;
        self.next_handler_id += 1;
        id
    }

    /// Allocate a fresh callback id for a Java functional-interface bridge.
    ///
    /// Ids are unique within a single plugin load; reset to 1 on each plugin reload.
    pub(crate) fn next_callback_id(&mut self) -> i64 {
        let id = self.next_callback_id;
        self.next_callback_id += 1;
        id
    }
}

/// Singleton Ctx storage.
///
/// `None` between `plugin_on_disable` and the next `plugin_init`. `install` refuses to overwrite an
/// existing `Some` so that reload-shutdown-then-init is the only legal init path.
static CTX: Mutex<Option<Ctx>> = Mutex::new(None);

/// Returned by [`install`] when called while `CTX` is already `Some`.
pub(crate) struct AlreadyInitialized;

/// Install `ctx` as the live singleton.
pub(crate) fn install(ctx: Ctx) -> Result<(), AlreadyInitialized> {
    let mut guard = CTX.lock().unwrap();
    if guard.is_some() {
        return Err(AlreadyInitialized);
    }
    *guard = Some(ctx);
    Ok(())
}

/// Drop the live singleton, if any. Idempotent.
pub(crate) fn uninstall() {
    *CTX.lock().unwrap() = None;
}

/// Run `body` with mutable access to the live `Ctx`.
///
/// Returns `None` if Ctx is not initialized. The lock is held for the duration of `body`, so `body`
/// must not invoke user closures or anything else that may re-enter `Ctx`.
pub(crate) fn with_ctx<R>(body: impl FnOnce(&mut Ctx) -> R) -> Option<R> {
    let mut guard = CTX.lock().unwrap();
    guard.as_mut().map(body)
}

/// Look up `name` in the Ctx-resident class cache, populating it on miss.
///
/// Returns a fresh `JClass<'local>` local ref derived from the cached global; the caller can use
/// it for the duration of the current JNI frame. The global stays in the cache for the lifetime
/// of the plugin load, so subsequent lookups for the same name skip the `FindClass` call entirely.
///
/// Names must be valid JVM class descriptors (e.g. `org/bukkit/entity/Player`). Invalid input
/// (interior NUL or non-modified-UTF-8) panics, since every call site passes a compile-time
/// literal -- bad input would be a build-time bug to fix.
pub(crate) fn cached_class<'local>(
    env: &mut Env<'local>,
    name: &'static str,
) -> jni::errors::Result<JClass<'local>> {
    // Cache hit: derive a fresh local from the cached global under the lock, return it.
    let hit = with_ctx(|c| -> jni::errors::Result<Option<JClass<'local>>> {
        match c.jni_cache.get(name) {
            Some(global) => Ok(Some(env.new_local_ref(global)?)),
            None => Ok(None),
        }
    })
    .expect("Ctx installed during plugin_init")?;
    if let Some(local) = hit {
        return Ok(local);
    }
    // Miss: find the class, install a global, return the original local.
    let cstring = CString::new(name).expect("class-name literal contains interior NUL byte");
    let jni_str =
        JNIStr::from_cstr(&cstring).expect("class-name literal is not valid modified UTF-8");
    let class_local = env.find_class(jni_str)?;
    let class_global = env.new_global_ref(&class_local)?;
    with_ctx(|c| {
        c.jni_cache.insert(name, class_global);
    });
    Ok(class_local)
}

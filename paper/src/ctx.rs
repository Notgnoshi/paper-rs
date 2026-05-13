use std::collections::HashMap;
use std::sync::Mutex;

use jni::objects::JObject;
use jni::refs::Global;

use crate::callbacks::BiConsumerFn;
use crate::dispatch::{CommandHandler, EventHandler};

/// Reload-scoped state for a Paper plugin.
///
/// [`Ctx`] is the single consolidated home for state that needs to outlive an individual JNI
/// dispatch call but not survive a `/reload`. Born in `core_init`, dropped in `core_shutdown`.
///
/// The Ctx is stored in a global static, but its lifetime is scoped by the plugin initialization
/// and shutdown.
///
/// User plugin code does not see `Ctx` directly; access is through `crate::Api` helpers.
pub(crate) struct Ctx {
    /// JNI global reference to the Java plugin object. Used wherever a registration sites needs
    /// to pass the plugin into a Bukkit call (event subscription, command registration, listener
    /// unregistration).
    pub(crate) plugin: Global<JObject<'static>>,
    /// Bukkit `Command` instances we've registered with the CommandMap. Drained at shutdown so
    /// the CommandMap doesn't retain stale handlers across `/reload`.
    pub(crate) registered_commands: Vec<Global<JObject<'static>>>,
    pub(crate) event_handlers: HashMap<i64, EventHandler>,
    pub(crate) command_handlers: HashMap<i64, CommandHandler>,
    pub(crate) callbacks: HashMap<i64, BiConsumerFn>,
    /// Cached `MiniMessage` singleton. Lazy-initialized on first use, since not every plugin
    /// touches MiniMessage.
    pub(crate) mini_message: Option<Global<JObject<'static>>>,
    next_handler_id: i64,
    next_callback_id: i64,
}

impl Ctx {
    pub(crate) fn new(plugin: Global<JObject<'static>>) -> Self {
        Self {
            plugin,
            registered_commands: Vec::new(),
            event_handlers: HashMap::new(),
            command_handlers: HashMap::new(),
            callbacks: HashMap::new(),
            mini_message: None,
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
/// `None` between `core_shutdown` and the next `core_init`. `install` refuses to overwrite an
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

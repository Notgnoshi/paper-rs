use std::sync::Mutex;

/// Reload-scoped state for a Paper plugin.
///
/// [`Ctx`] is the single consolidated home for state that needs to outlive an individual JNI
/// dispatch call but not survive a `/reload`. Born in `core_init`, dropped in `core_shutdown`.
///
/// The storage is a single `static CTX: Mutex<Option<Ctx>>` in this module. Stage 1 leaves `Ctx`
/// empty; subsequent stages migrate the existing scattered `Mutex<Option<HashMap<...>>>` statics
/// (handler maps, callback registry, plugin Global ref, registered commands, MiniMessage singleton)
/// into `Ctx` fields one at a time.
///
/// User plugin code does not see `Ctx` directly; access is through `crate::Api` helpers.
pub(crate) struct Ctx {}

impl Ctx {
    pub(crate) fn new() -> Self {
        Self {}
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

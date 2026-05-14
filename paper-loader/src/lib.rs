//! Paper plugin loader.
//!
//! Java can't unload a native library once it's been dlopened, so we load a stable plugin loader,
//! and then dlopen the actual plugin here, so that we can close it and reload it on demand.
use std::ffi::OsStr;
use std::mem::size_of;
use std::sync::Arc;

use arc_swap::ArcSwapOption;
use jni::errors::{Error, ThrowRuntimeExAndDefault};
use jni::objects::{JClass, JObject, JString};
use jni::sys::{JNI_FALSE, jboolean, jlong, jobject, jobjectArray};
use jni::{Env, EnvUnowned};
use libloading::{Library, Symbol};
use paper::{FnTable, PLUGIN_ABI_VERSION};

mod logger;

/// A loaded plugin: the `dlopen`-managed library and the function-pointer table it exported
/// at `papermc_plugin_init` time.
struct LoadedPlugin {
    _lib: Library,
    api: *const FnTable,
}

// SAFETY: `api` points at a `static FnTable` inside the dlopen'd plugin. The table is constructed
// once at plugin init time and never mutated; all entries are bare `extern "C" fn` pointers, which
// are `Send + Sync`. The `Library` is `Send + Sync` already.
unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

/// Active plugin, or `None` between shutdown and the next init.
///
/// `ArcSwap` lets dispatch threads grab the current `Arc<LoadedPlugin>` with a single atomic load
/// without lock contention.
static LOADED_PLUGIN: ArcSwapOption<LoadedPlugin> = ArcSwapOption::const_empty();

/// Symbol the loader looks up in the plugin .so
const PLUGIN_INIT_SYMBOL: &[u8] = b"papermc_plugin_init";

type PluginInitFn =
    unsafe extern "C" fn(*mut jni::sys::JNIEnv, jni::sys::jobject) -> *const FnTable;

/// Drop the cached `LoadedPlugin`. The dlclose that frees the underlying .so won't happen until
/// the last in-flight dispatch releases its `Arc` reference.
///
/// Caller is responsible for having invoked the plugin's `shutdown` first so the plugin can
/// release its JNI globals before the mapping disappears.
fn unload_plugin() {
    LOADED_PLUGIN.store(None);
}

fn load_plugin(
    path: &str,
    env_ptr: *mut jni::sys::JNIEnv,
    plugin_ptr: jni::sys::jobject,
) -> Result<*const FnTable, String> {
    let lib = unsafe { Library::new(OsStr::new(path)) }
        .map_err(|e| format!("dlopen({path}) failed: {e}"))?;
    let init: Symbol<PluginInitFn> = unsafe {
        lib.get(PLUGIN_INIT_SYMBOL)
            .map_err(|e| format!("dlsym(papermc_plugin_init) failed: {e}"))?
    };
    let api_ptr = unsafe { init(env_ptr, plugin_ptr) };
    if api_ptr.is_null() {
        return Err("papermc_plugin_init returned null".into());
    }
    // Safety: trust the plugin to populate abi_version/size correctly.
    let api = unsafe { &*api_ptr };
    if api.abi_version != PLUGIN_ABI_VERSION {
        return Err(format!(
            "plugin ABI version {} does not match loader's {PLUGIN_ABI_VERSION}",
            api.abi_version,
        ));
    }
    if (api.size as usize) < size_of::<FnTable>() {
        return Err(format!(
            "plugin FnTable size {} smaller than loader's {}",
            api.size,
            size_of::<FnTable>(),
        ));
    }
    LOADED_PLUGIN.store(Some(Arc::new(LoadedPlugin {
        _lib: lib,
        api: api_ptr,
    })));
    Ok(api_ptr)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_papermc_RustPlugin_on_1enable<'local>(
    mut unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    plugin_path: JString<'local>,
    plugin: JObject<'local>,
) {
    unowned
        .with_env(|env: &mut Env<'local>| -> jni::errors::Result<()> {
            // Wire the tracing subscriber up first; any tracing below this point reaches Java.
            // Earlier events (before this line) would land in /dev/null since the subscriber
            // isn't installed yet, so there's nothing to log here.
            if let Err(e) = logger::install(env) {
                eprintln!("paper-loader: logger install failed: {e}");
            }
            tracing::debug!("paper-loader: init entered");
            let path = plugin_path.try_to_string(env)?;
            // Atomically take ownership of any stale plugin. Dispatch threads hitting the
            // swap-to-None window still hold guards on the old Arc, so its `Library` won't
            // dlclose until they drop.
            let stale = LOADED_PLUGIN.swap(None);
            if let Some(loaded) = stale.as_ref() {
                tracing::warn!(
                    "paper-loader: stale plugin present; running shutdown before re-load"
                );
                let _ = unsafe { ((*loaded.api).shutdown)(env.get_raw()) };
                drop(stale);
            }
            tracing::debug!("paper-loader: dlopen({path})");
            let _api_ptr = load_plugin(&path, env.get_raw(), plugin.as_raw()).map_err(|msg| {
                tracing::error!("paper-loader: load_plugin failed: {msg}");
                let _ = env.throw(msg);
                Error::JavaException
            })?;
            tracing::info!("paper-loader: {path} init complete");
            Ok(())
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_papermc_RustPlugin_on_1disable<'local>(
    mut unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    tracing::info!("paper-loader: shutdown entered");
    let _ = unowned
        .with_env(|env: &mut Env<'local>| -> jni::errors::Result<()> {
            let guard = LOADED_PLUGIN.load();
            if let Some(loaded) = guard.as_ref() {
                tracing::info!("paper-loader: calling plugin shutdown");
                let _ = unsafe { ((*loaded.api).shutdown)(env.get_raw()) };
                drop(guard);
                tracing::debug!("paper-loader: dropping plugin library (dlclose may be deferred)");
                unload_plugin();
                tracing::debug!("paper-loader: unload complete");
            } else {
                tracing::warn!("paper-loader: no plugin to shutdown");
            }
            // Drop the dispatcher class Global so the unloading plugin's ClassLoader can be GC'd.
            // Tracing events between here and the next install will no-op silently.
            logger::shutdown();
            Ok(())
        })
        .into_outcome();
}

// JNI native-method export: callers are the JVM, which is responsible for jobject validity. The
// allow covers the raw `jobject`/`jobjectArray` params that get forwarded into the plugin's
// `unsafe extern "C"` dispatch entrypoint.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_papermc_RustPlugin_dispatch_1event<'local>(
    unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    handler_id: jlong,
    event: jobject,
) {
    let guard = LOADED_PLUGIN.load();
    let Some(loaded) = guard.as_ref() else { return };
    // Forward without entering with_env: the plugin's dispatch_event will set up
    // its own EnvUnowned/with_env from the raw pointer.
    let raw_env = EnvUnowned::into_raw(unowned);
    unsafe { ((*loaded.api).dispatch_event)(raw_env, handler_id, event) };
    // `guard` drops here; the `Arc<LoadedPlugin>` only fully releases once all such guards have
    // returned, so the .so mapping can't disappear out from under an in-flight dispatch.
}

// JNI native-method export: jobject params come from the JVM and are forwarded into the plugin.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_papermc_RustPlugin_dispatch_1command<'local>(
    unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jboolean {
    let guard = LOADED_PLUGIN.load();
    let Some(loaded) = guard.as_ref() else {
        return JNI_FALSE;
    };
    let raw_env = EnvUnowned::into_raw(unowned);
    unsafe { ((*loaded.api).dispatch_command)(raw_env, handler_id, sender, args) }
}

// JNI native-method export: jobject params come from the JVM and are forwarded into the plugin.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_papermc_RustPlugin_dispatch_1tab_1complete<'local>(
    unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jobject {
    let guard = LOADED_PLUGIN.load();
    let Some(loaded) = guard.as_ref() else {
        return std::ptr::null_mut();
    };
    let raw_env = EnvUnowned::into_raw(unowned);
    unsafe { ((*loaded.api).dispatch_tab_complete)(raw_env, handler_id, sender, args) }
}

/// Bridge for `RustDialogActionCallback.bridgeDispatch(long id, Object t, Object u)`.
//
// JNI native-method export: jobject params come from the JVM and are forwarded into the plugin.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_papermc_RustDialogActionCallback_bridgeDispatch<'local>(
    unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    id: jlong,
    t: jobject,
    u: jobject,
) {
    let guard = LOADED_PLUGIN.load();
    let Some(loaded) = guard.as_ref() else { return };
    let raw_env = EnvUnowned::into_raw(unowned);
    unsafe { ((*loaded.api).dispatch_bi_consumer)(raw_env, id, t, u) };
}

/// Bridge for `RustDialogActionCallback.bridgeDrop(long id)`, called from Cleaner.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_papermc_RustDialogActionCallback_bridgeDrop<'local>(
    _unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    id: jlong,
) {
    let guard = LOADED_PLUGIN.load();
    let Some(loaded) = guard.as_ref() else { return };
    unsafe { ((*loaded.api).drop_callback)(id) };
}

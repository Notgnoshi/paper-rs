//! Paper plugin loader.
//!
//! Java can't unload a native library once it's been dlopened, so we load a stable plugin loader,
//! and then dlopen the actual plugin here, so that we can close it and reload it on demand.
use std::ffi::OsStr;
use std::mem::size_of;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

use jni::errors::{Error, ThrowRuntimeExAndDefault};
use jni::objects::{JClass, JObject, JString};
use jni::sys::{JNI_FALSE, jboolean, jlong, jobject, jobjectArray};
use jni::{Env, EnvUnowned};
use libloading::{Library, Symbol};
use paper::{CORE_ABI_VERSION, CoreApi};

static CORE_LIB: Mutex<Option<Library>> = Mutex::new(None);
static CORE_API: AtomicPtr<CoreApi> = AtomicPtr::new(std::ptr::null_mut());

/// Symbol the loader looks up in the core .so
const CORE_INIT_SYMBOL: &[u8] = b"paper_core_init";

type CoreInitFn = unsafe extern "C" fn(*mut jni::sys::JNIEnv, jni::sys::jobject) -> *const CoreApi;

/// Drop the cached library + API pointer. Caller must have already invoked
/// core's shutdown (so the core can release JNI globals before its .so unloads).
fn unload_core() {
    CORE_API.store(std::ptr::null_mut(), Ordering::SeqCst);
    *CORE_LIB.lock().unwrap() = None;
}

fn load_core(
    path: &str,
    env_ptr: *mut jni::sys::JNIEnv,
    plugin_ptr: jni::sys::jobject,
) -> Result<*const CoreApi, String> {
    let lib = unsafe { Library::new(OsStr::new(path)) }
        .map_err(|e| format!("dlopen({path}) failed: {e}"))?;
    let init: Symbol<CoreInitFn> = unsafe {
        lib.get(CORE_INIT_SYMBOL)
            .map_err(|e| format!("dlsym(paper_core_init) failed: {e}"))?
    };
    let api_ptr = unsafe { init(env_ptr, plugin_ptr) };
    if api_ptr.is_null() {
        return Err("paper_core_init returned null".into());
    }
    // Safety: trust the core to populate abi_version/size correctly.
    let api = unsafe { &*api_ptr };
    if api.abi_version != CORE_ABI_VERSION {
        return Err(format!(
            "core ABI version {} does not match loader's {CORE_ABI_VERSION}",
            api.abi_version,
        ));
    }
    if (api.size as usize) < size_of::<CoreApi>() {
        return Err(format!(
            "core CoreApi size {} smaller than loader's {}",
            api.size,
            size_of::<CoreApi>(),
        ));
    }
    *CORE_LIB.lock().unwrap() = Some(lib);
    CORE_API.store(api_ptr as *mut CoreApi, Ordering::SeqCst);
    Ok(api_ptr)
}

fn current_api() -> Option<*const CoreApi> {
    let p = CORE_API.load(Ordering::SeqCst);
    if p.is_null() {
        None
    } else {
        Some(p as *const CoreApi)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_paperrs_shim_PaperRs_init<'local>(
    mut unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    core_path: JString<'local>,
    plugin: JObject<'local>,
) {
    eprintln!("paper-loader: init entered");
    unowned
        .with_env(|env: &mut Env<'local>| -> jni::errors::Result<()> {
            let path = core_path.try_to_string(env)?;
            if let Some(api) = current_api() {
                eprintln!("paper-loader: stale CoreApi present; running shutdown before re-load");
                let _ = unsafe { ((*api).shutdown)(env.get_raw()) };
                unload_core();
            }
            eprintln!("paper-loader: dlopen({path})");
            let _api_ptr = load_core(&path, env.get_raw(), plugin.as_raw()).map_err(|msg| {
                eprintln!("paper-loader: load_core failed: {msg}");
                let _ = env.throw(msg);
                Error::JavaException
            })?;
            eprintln!("paper-loader: init complete");
            Ok(())
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_paperrs_shim_PaperRs_shutdown<'local>(
    mut unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    eprintln!("paper-loader: shutdown entered");
    let _ = unowned
        .with_env(|env: &mut Env<'local>| -> jni::errors::Result<()> {
            if let Some(api) = current_api() {
                eprintln!("paper-loader: calling core shutdown");
                let _ = unsafe { ((*api).shutdown)(env.get_raw()) };
                eprintln!("paper-loader: dropping core library (dlclose)");
                unload_core();
                eprintln!("paper-loader: unload complete");
            } else {
                eprintln!("paper-loader: no CoreApi to shutdown");
            }
            Ok(())
        })
        .into_outcome();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_paperrs_shim_PaperRs_dispatchEvent<'local>(
    unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    handler_id: jlong,
    event: jobject,
) {
    let Some(api) = current_api() else { return };
    // Forward without entering with_env: core's dispatch_event will set up
    // its own EnvUnowned/with_env from the raw pointer.
    let raw_env = EnvUnowned::into_raw(unowned);
    unsafe { ((*api).dispatch_event)(raw_env, handler_id, event) };
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_paperrs_shim_PaperRs_dispatchCommand<'local>(
    unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jboolean {
    let Some(api) = current_api() else {
        return JNI_FALSE;
    };
    let raw_env = EnvUnowned::into_raw(unowned);
    unsafe { ((*api).dispatch_command)(raw_env, handler_id, sender, args) }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_paperrs_shim_PaperRs_dispatchTabComplete<'local>(
    unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jobject {
    let Some(api) = current_api() else {
        return std::ptr::null_mut();
    };
    let raw_env = EnvUnowned::into_raw(unowned);
    unsafe { ((*api).dispatch_tab_complete)(raw_env, handler_id, sender, args) }
}

/// Bridge for `RustDialogActionCallback.bridgeDispatch(long id, Object t, Object u)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_paperrs_shim_RustDialogActionCallback_bridgeDispatch<'local>(
    unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    id: jlong,
    t: jobject,
    u: jobject,
) {
    let Some(api) = current_api() else { return };
    let raw_env = EnvUnowned::into_raw(unowned);
    unsafe { ((*api).dispatch_bi_consumer)(raw_env, id, t, u) };
}

/// Bridge for `RustDialogActionCallback.bridgeDrop(long id)`, called from Cleaner.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_paperrs_shim_RustDialogActionCallback_bridgeDrop<'local>(
    _unowned: EnvUnowned<'local>,
    _class: JClass<'local>,
    id: jlong,
) {
    let Some(api) = current_api() else { return };
    unsafe { ((*api).drop_callback)(id) };
}

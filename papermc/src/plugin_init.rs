use std::mem::size_of;

use jni::Env;
use jni::objects::JObject;
use tracing::warn;

use crate::builder::PluginBuilder;
use crate::{FnTable, PLUGIN_ABI_VERSION, callbacks, ctx, dispatch, ffi, registration};

/// The static `FnTable` returned by every `papermc_plugin_init` call.
static FN_TABLE: FnTable = FnTable {
    abi_version: PLUGIN_ABI_VERSION,
    size: size_of::<FnTable>() as u32,
    shutdown: plugin_shutdown,
    dispatch_event: dispatch::dispatch_event,
    dispatch_command: dispatch::dispatch_command,
    dispatch_tab_complete: dispatch::dispatch_tab_complete,
    dispatch_bi_consumer: callbacks::dispatch_bi_consumer,
    drop_callback: callbacks::drop_callback,
};

unsafe extern "C" fn plugin_shutdown(env: *mut jni::sys::JNIEnv) -> i32 {
    let result = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<()> {
        if let Err(e) = registration::unregister_commands(env) {
            warn!("unregister_commands failed: {e}");
            env.exception_clear();
        }
        if let Err(e) = registration::unregister_all_listeners(env) {
            warn!("unregister_all_listeners failed: {e}");
            env.exception_clear();
        }
        // Drops any static state initialized during plugin runtime along with any captured JNI globals.
        ctx::uninstall();
        Ok(())
    });
    match result {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

/// The single helper plugin authors call from their `papermc_plugin_init` C-ABI export.
///
/// Builds a `PluginBuilder`, runs the user's `build` closure to register handlers, and returns the
/// static `FnTable` the loader will dispatch through.
///
/// Returns a null pointer if the build closure returned `Err`. papermc-loader maps a null return to a
/// Java `RuntimeException`, aborting plugin init cleanly with the underlying exception surfaced via
/// Bukkit's normal error path.
//
// `plugin_init` is invoked from a plugin's C-ABI `papermc_plugin_init` symbol with raw pointers
// handed to it by the JVM. JNI's calling convention is the contract for those pointers being
// valid; null is null-checked inside [`ffi::bridge`]. Keeping this function safe at the Rust level
// lets plugin authors write `papermc::plugin_init(env, plugin, ...)` without an unsafe wrapper at
// every call site.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn plugin_init<F>(
    env: *mut jni::sys::JNIEnv,
    plugin: jni::sys::jobject,
    build: F,
) -> *const FnTable
where
    F: FnOnce(&mut PluginBuilder<'_, '_>) -> eyre::Result<()>,
{
    let result = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<()> {
        let plugin_obj = unsafe { JObject::from_raw(env, plugin) };
        let plugin_global = env.new_global_ref(&plugin_obj)?;
        if ctx::install(ctx::Ctx::new(plugin_global)).is_err() {
            eyre::bail!("papermc_plugin_init: Ctx already initialized (prior shutdown missing)");
        }
        let mut builder = PluginBuilder::new(env);
        build(&mut builder)?;
        Ok(())
    });
    match result {
        Ok(()) => &FN_TABLE,
        Err(_) => std::ptr::null(),
    }
}

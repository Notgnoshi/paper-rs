use std::mem::size_of;

use jni::objects::JObject;
use jni::{Env, EnvUnowned};
use tracing::warn;

use crate::builder::PluginBuilder;
use crate::{CORE_ABI_VERSION, CoreApi, callbacks, ctx, dispatch, logger, registration};

/// The static CoreApi table returned by every `paper_core_init` call.
static CORE_API: CoreApi = CoreApi {
    abi_version: CORE_ABI_VERSION,
    size: size_of::<CoreApi>() as u32,
    shutdown: core_shutdown,
    dispatch_event: dispatch::dispatch_event,
    dispatch_command: dispatch::dispatch_command,
    dispatch_tab_complete: dispatch::dispatch_tab_complete,
    dispatch_bi_consumer: callbacks::dispatch_bi_consumer,
    drop_callback: callbacks::drop_callback,
};

unsafe extern "C" fn core_shutdown(env: *mut jni::sys::JNIEnv) -> i32 {
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let outcome = unowned.with_env(|env: &mut Env<'_>| -> jni::errors::Result<()> {
        if let Err(e) = registration::unregister_commands(env) {
            warn!("unregister_commands failed: {e}");
            env.exception_clear();
        }
        if let Err(e) = registration::unregister_all_listeners(env) {
            warn!("unregister_all_listeners failed: {e}");
            env.exception_clear();
        }
        logger::shutdown_logger();
        // Drops any static state initialized during plugin runtime along with any captured JNI globals.
        ctx::uninstall();
        Ok(())
    });
    match outcome.into_outcome() {
        jni::Outcome::Ok(_) => 0,
        jni::Outcome::Err(e) => {
            warn!("core_shutdown failed: {e}");
            1
        }
        jni::Outcome::Panic(_) => {
            warn!("core_shutdown panicked");
            2
        }
    }
}

/// The single helper plugin authors call from their `paper_core_init` C-ABI export.
///
/// Builds a `PluginBuilder`, runs the user's `build` closure to register handlers, and returns the
/// static `CoreApi` table the loader will dispatch through.
///
/// Returns a null pointer if the build closure returned `Err`. paper-loader maps a null return to a
/// Java `RuntimeException`, aborting plugin init cleanly with the underlying exception surfaced via
/// Bukkit's normal error path.
pub fn core_init<F>(
    env: *mut jni::sys::JNIEnv,
    plugin: jni::sys::jobject,
    build: F,
) -> *const CoreApi
where
    F: FnOnce(&mut PluginBuilder<'_, '_>) -> jni::errors::Result<()>,
{
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let outcome = unowned
        .with_env(|env: &mut Env<'_>| -> jni::errors::Result<()> {
            let plugin_obj = unsafe { JObject::from_raw(env, plugin) };
            let plugin_global = env.new_global_ref(&plugin_obj)?;
            if ctx::install(ctx::Ctx::new(plugin_global)).is_err() {
                let _ =
                    env.throw("paper_core_init: Ctx already initialized (prior shutdown missing)");
                return Err(jni::errors::Error::JavaException);
            }
            logger::install_logger(env)?;
            let mut builder = PluginBuilder::new(env);
            build(&mut builder)
        })
        .into_outcome();
    match outcome {
        jni::Outcome::Ok(_) => &CORE_API,
        jni::Outcome::Err(e) => {
            tracing::error!("paper_core_init failed: {e}");
            std::ptr::null()
        }
        jni::Outcome::Panic(p) => {
            tracing::error!("paper_core_init panicked: {p:?}");
            std::ptr::null()
        }
    }
}

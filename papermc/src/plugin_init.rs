use std::mem::size_of;

use jni::Env;
use jni::objects::JObject;
use tracing::warn;

use crate::api::Api;
use crate::plugin::Plugin;
use crate::setup_api::SetupApi;
use crate::{FnTable, PLUGIN_ABI_VERSION, callbacks, ctx, dispatch, ffi, logger, registration};

/// The static `FnTable` returned by every `papermc_plugin_init` call.
static FN_TABLE: FnTable = FnTable {
    abi_version: PLUGIN_ABI_VERSION,
    size: size_of::<FnTable>() as u32,
    on_disable: plugin_on_disable,
    dispatch_event: dispatch::dispatch_event,
    dispatch_command: dispatch::dispatch_command,
    dispatch_tab_complete: dispatch::dispatch_tab_complete,
    dispatch_bi_consumer: callbacks::dispatch_bi_consumer,
    drop_callback: callbacks::drop_callback,
};

unsafe extern "C" fn plugin_on_disable(env: *mut jni::sys::JNIEnv) -> i32 {
    let result = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<()> {
        // Invoke the user's `Plugin::on_disable` (if a typed plugin was installed via `init::<P>`)
        // before tearing down anything else, so the user code still sees a live Ctx and JNI env.
        let plugin_and_fn = ctx::with_ctx(|c| {
            let plugin = c.rust_plugin.take();
            let on_disable = c.on_disable_fn.take();
            plugin.zip(on_disable)
        })
        .flatten();
        if let Some((mut plugin_box, on_disable)) = plugin_and_fn {
            let mut api = Api::new(env);
            if let Err(e) = on_disable(plugin_box.as_mut(), &mut api) {
                warn!("Plugin::on_disable failed: {e}");
            }
            drop(plugin_box);
        }
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
        // Release the dispatcher-class Global this cdylib was holding. Tracing events emitted from
        // this cdylib between here and the next `init` no-op silently.
        logger::unbind_dispatcher();
        Ok(())
    });
    match result {
        Ok(()) => 0,
        Err(_) => 1,
    }
}

/// Plugin init driver.
///
/// Plugin authors write a struct that implements [`Plugin`], then call this from their C-ABI
/// `papermc_plugin_init` export:
///
/// ```ignore
/// #[unsafe(no_mangle)]
/// pub extern "C" fn papermc_plugin_init(
///     env: *mut jni::sys::JNIEnv,
///     plugin: jni::sys::jobject,
/// ) -> *const papermc::FnTable {
///     papermc::init::<MyPlugin>(env, plugin)
/// }
/// ```
///
/// Returns a null pointer if `P::on_enable` returned `Err` or the JNI bridge tripped.
//
// `init` is invoked from a plugin's C-ABI `papermc_plugin_init` symbol with raw pointers handed to
// it by the JVM. JNI's calling convention is the contract for those pointers being valid; null is
// null-checked inside [`ffi::bridge`]. Keeping this function safe at the Rust level lets plugin
// authors write `papermc::init::<MyPlugin>(env, plugin)` without an unsafe wrapper at the call
// site.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn init<P: Plugin>(env: *mut jni::sys::JNIEnv, plugin: jni::sys::jobject) -> *const FnTable {
    let result = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<()> {
        // Install this cdylib's tracing subscriber and bind the dispatcher class, so plugin-side
        // `tracing::*` events reach Java. The install is `Once`-guarded; `bind_dispatcher` runs
        // every enable so the cached class doesn't pin a stale ClassLoader after `/reload`.
        logger::install_subscriber(env.get_java_vm()?);
        if let Err(e) = logger::bind_dispatcher(env) {
            eprintln!("papermc::init: bind_dispatcher failed: {e}");
        }
        let plugin_obj = unsafe { JObject::from_raw(env, plugin) };
        let plugin_global = env.new_global_ref(&plugin_obj)?;
        if ctx::install(ctx::Ctx::new(plugin_global)).is_err() {
            eyre::bail!("papermc::init: Ctx already initialized (prior shutdown missing)");
        }
        let api = Api::new(env);
        let mut setup = SetupApi::<P>::new(api);
        let plugin_state = P::on_enable(&mut setup)?;
        ctx::with_ctx(|c| {
            c.rust_plugin = Some(Box::new(plugin_state));
            c.on_disable_fn = Some(Box::new(|any, api| {
                let p = any
                    .downcast_mut::<P>()
                    .expect("plugin type mismatch in on_disable trampoline");
                P::on_disable(p, api)
            }));
        });
        Ok(())
    });
    match result {
        Ok(()) => &FN_TABLE,
        Err(_) => std::ptr::null(),
    }
}

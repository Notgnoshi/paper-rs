use std::sync::Arc;

use jni::objects::JObject;
use jni::sys::{JNIEnv, jlong, jobject};
use jni::{Env, EnvUnowned};
use tracing::warn;

use crate::api::Api;
use crate::ctx;

/// A Rust closure backing a two-argument Java functional interface (e.g.,
/// `DialogActionCallback.accept(DialogResponseView, Audience)`).
pub(crate) type BiConsumerFn =
    Arc<dyn for<'a> Fn(&mut Api<'_, 'a>, &JObject<'a>, &JObject<'a>) + Send + Sync>;

/// Trampoline target for the `RustDialogActionCallback.bridgeDispatch` native method.
///
/// paper-loader's stable JNI symbol forwards here via the `CoreApi::dispatch_bi_consumer`
/// function pointer.
pub(crate) unsafe extern "C" fn dispatch_bi_consumer(
    env_raw: *mut JNIEnv,
    id: jlong,
    t: jobject,
    u: jobject,
) {
    let mut unowned = unsafe { EnvUnowned::from_raw(env_raw) };
    let _ = unowned
        .with_env(|env: &mut Env<'_>| -> jni::errors::Result<()> {
            let t_obj = unsafe { JObject::from_raw(env, t) };
            let u_obj = unsafe { JObject::from_raw(env, u) };
            let callback = ctx::with_ctx(|c| c.callbacks.get(&id).cloned()).flatten();
            let Some(callback) = callback else {
                warn!("no callback registered for id {id}");
                return Ok(());
            };
            let mut api = Api::new(env);
            callback(&mut api, &t_obj, &u_obj);
            Ok(())
        })
        .into_outcome();
}

/// Trampoline target for the `RustDialogActionCallback.bridgeDrop` native method.
///
/// Called from Java's Cleaner after the bridge instance is unreachable.
pub(crate) unsafe extern "C" fn drop_callback(id: jlong) {
    ctx::with_ctx(|c| {
        c.callbacks.remove(&id);
    });
}

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};

use jni::objects::JObject;
use jni::sys::{JNIEnv, jlong, jobject};
use jni::{Env, EnvUnowned};
use tracing::warn;

use crate::api::Api;

/// A Rust closure backing a two-argument Java functional interface (e.g.,
/// `DialogActionCallback.accept(DialogResponseView, Audience)`).
pub(crate) type BiConsumerFn =
    Box<dyn for<'a> Fn(&mut Api<'_, 'a>, &JObject<'a>, &JObject<'a>) + Send + Sync>;

static CALLBACKS: Mutex<Option<HashMap<i64, BiConsumerFn>>> = Mutex::new(None);
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

pub(crate) fn next_id() -> i64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

pub(crate) fn register(id: i64, callback: BiConsumerFn) {
    let mut guard = CALLBACKS.lock().unwrap();
    guard.get_or_insert_with(HashMap::new).insert(id, callback);
}

/// Drop the entire registry. Called from `core_shutdown` so closures (and any captured state)
/// get freed while the .so is still mapped.
pub(crate) fn clear() {
    *CALLBACKS.lock().unwrap() = None;
}

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
            let guard = CALLBACKS.lock().unwrap();
            let Some(map) = guard.as_ref() else {
                return Ok(());
            };
            let Some(callback) = map.get(&id) else {
                warn!("no callback registered for id {id}");
                return Ok(());
            };
            let t_obj = unsafe { JObject::from_raw(env, t) };
            let u_obj = unsafe { JObject::from_raw(env, u) };
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
    let mut guard = CALLBACKS.lock().unwrap();
    if let Some(map) = guard.as_mut() {
        map.remove(&id);
    }
}

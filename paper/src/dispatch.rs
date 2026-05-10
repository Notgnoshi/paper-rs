use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};

use jni::objects::{JObject, JObjectArray, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jlong, jobject, jobjectArray};
use jni::{Env, EnvUnowned};
use tracing::warn;

pub(crate) type EventHandler = Box<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>) + Send + Sync>;
pub(crate) type CommandHandler =
    Box<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>, &[String]) -> bool + Send + Sync>;

static EVENT_HANDLERS: Mutex<Option<HashMap<i64, EventHandler>>> = Mutex::new(None);
static COMMAND_HANDLERS: Mutex<Option<HashMap<i64, CommandHandler>>> = Mutex::new(None);
static NEXT_HANDLER_ID: AtomicI64 = AtomicI64::new(1);

pub(crate) fn next_handler_id() -> i64 {
    NEXT_HANDLER_ID.fetch_add(1, Ordering::SeqCst)
}

pub(crate) fn insert_event_handler(id: i64, handler: EventHandler) {
    let mut guard = EVENT_HANDLERS.lock().unwrap();
    guard.get_or_insert_with(HashMap::new).insert(id, handler);
}

pub(crate) fn insert_command_handler(id: i64, handler: CommandHandler) {
    let mut guard = COMMAND_HANDLERS.lock().unwrap();
    guard.get_or_insert_with(HashMap::new).insert(id, handler);
}

/// Drop both handler maps.
///
/// Called from `core_shutdown` so closures (and any captured state) get freed while the .so is
/// still mapped.
pub(crate) fn clear_handlers() {
    *EVENT_HANDLERS.lock().unwrap() = None;
    *COMMAND_HANDLERS.lock().unwrap() = None;
}

pub(crate) unsafe extern "C" fn dispatch_event(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    event: jobject,
) {
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let _ = unowned
        .with_env(|env: &mut Env<'_>| -> jni::errors::Result<()> {
            let map_guard = EVENT_HANDLERS.lock().unwrap();
            let Some(map) = map_guard.as_ref() else {
                return Ok(());
            };
            let Some(handler) = map.get(&handler_id) else {
                warn!("no event handler registered for id {handler_id}");
                return Ok(());
            };
            let event_obj = unsafe { JObject::from_raw(env, event) };
            handler(env, &event_obj);
            Ok(())
        })
        .into_outcome();
}

pub(crate) unsafe extern "C" fn dispatch_command(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jboolean {
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let outcome = unowned.with_env(|env: &mut Env<'_>| -> jni::errors::Result<bool> {
        let map_guard = COMMAND_HANDLERS.lock().unwrap();
        let Some(map) = map_guard.as_ref() else {
            return Ok(false);
        };
        let Some(handler) = map.get(&handler_id) else {
            warn!("no command handler registered for id {handler_id}");
            return Ok(false);
        };
        let sender_obj = unsafe { JObject::from_raw(env, sender) };
        let args_arr = unsafe { JObjectArray::<JString>::from_raw(env, args) };
        let args_vec = read_string_array(env, &args_arr)?;
        let result = handler(env, &sender_obj, &args_vec);
        Ok(result)
    });
    match outcome.into_outcome() {
        jni::Outcome::Ok(b) => {
            if b {
                JNI_TRUE
            } else {
                JNI_FALSE
            }
        }
        _ => JNI_FALSE,
    }
}

pub(crate) unsafe extern "C" fn dispatch_tab_complete(
    _env: *mut jni::sys::JNIEnv,
    _handler_id: jlong,
    _sender: jobject,
    _args: jobjectArray,
) -> jobject {
    std::ptr::null_mut()
}

fn read_string_array(
    env: &mut Env<'_>,
    arr: &JObjectArray<'_, JString>,
) -> jni::errors::Result<Vec<String>> {
    let len = arr.len(env)?;
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let elem = arr.get_element(env, i)?;
        let s = elem.try_to_string(env)?;
        out.push(s);
    }
    Ok(out)
}

use std::sync::Arc;

use jni::objects::{JObject, JObjectArray, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jlong, jobject, jobjectArray};
use jni::{Env, EnvUnowned};
use tracing::warn;

use crate::ctx;

pub(crate) type EventHandler = Arc<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>) + Send + Sync>;
pub(crate) type CommandHandler =
    Arc<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>, &[String]) -> bool + Send + Sync>;

pub(crate) unsafe extern "C" fn dispatch_event(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    event: jobject,
) {
    let mut unowned = unsafe { EnvUnowned::from_raw(env) };
    let _ = unowned
        .with_env(|env: &mut Env<'_>| -> jni::errors::Result<()> {
            let event_obj = unsafe { JObject::from_raw(env, event) };
            let handler = ctx::with_ctx(|c| c.event_handlers.get(&handler_id).cloned()).flatten();
            let Some(handler) = handler else {
                warn!("no event handler registered for id {handler_id}");
                return Ok(());
            };
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
        let sender_obj = unsafe { JObject::from_raw(env, sender) };
        let args_arr = unsafe { JObjectArray::<JString>::from_raw(env, args) };
        let args_vec = read_string_array(env, &args_arr)?;
        let handler = ctx::with_ctx(|c| c.command_handlers.get(&handler_id).cloned()).flatten();
        let Some(handler) = handler else {
            warn!("no command handler registered for id {handler_id}");
            return Ok(false);
        };
        Ok(handler(env, &sender_obj, &args_vec))
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

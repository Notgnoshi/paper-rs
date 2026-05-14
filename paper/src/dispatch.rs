use std::sync::Arc;

use jni::Env;
use jni::objects::{JObject, JObjectArray, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jlong, jobject, jobjectArray};
use tracing::warn;

use crate::{ctx, ffi};

pub(crate) type EventHandler = Arc<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>) + Send + Sync>;
pub(crate) type CommandHandler =
    Arc<dyn for<'a> Fn(&mut Env<'a>, &JObject<'a>, &[String]) -> bool + Send + Sync>;

pub(crate) unsafe extern "C" fn dispatch_event(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    event: jobject,
) {
    let _ = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<()> {
        let event_obj = unsafe { JObject::from_raw(env, event) };
        let handler = ctx::with_ctx(|c| c.event_handlers.get(&handler_id).cloned()).flatten();
        let Some(handler) = handler else {
            warn!("no event handler registered for id {handler_id}");
            return Ok(());
        };
        handler(env, &event_obj);
        Ok(())
    });
}

pub(crate) unsafe extern "C" fn dispatch_command(
    env: *mut jni::sys::JNIEnv,
    handler_id: jlong,
    sender: jobject,
    args: jobjectArray,
) -> jboolean {
    let result = ffi::bridge(env, |env: &mut Env<'_>| -> eyre::Result<bool> {
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
    match result {
        Ok(true) => JNI_TRUE,
        Ok(false) => JNI_FALSE,
        Err(_) => JNI_FALSE,
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
    // Each `get_element` allocates a local JNI ref. JNI guarantees only 16 locals by default, so a
    // long argument list overflows the outer frame's allotment. Push a sized sub-frame so those
    // intermediates are released en masse when this helper returns
    env.with_local_frame(len + 4, |env| -> jni::errors::Result<Vec<String>> {
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let elem = arr.get_element(env, i)?;
            let s = elem.try_to_string(env)?;
            out.push(s);
        }
        Ok(out)
    })
}

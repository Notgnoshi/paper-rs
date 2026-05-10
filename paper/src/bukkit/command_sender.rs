use jni::objects::{JObject, JString, JValue};
use jni::strings::JNIStr;
use jni::{Env, jni_sig, jni_str};

use crate::api::Api;

/// Wrapper for an `org.bukkit.command.CommandSender` JNI reference.
///
/// `#[repr(transparent)]` over `JObject` so we can reinterpret a borrowed `&JObject` as a borrowed
/// `&CommandSender` at dispatch time. The reinterpret is gated by an `is_instance_of` check in
/// `wrap_ref` so a Bukkit contract change can't silently feed us the wrong class.
#[repr(transparent)]
pub struct CommandSender<'local> {
    obj: JObject<'local>,
}

const CLASS_NAME: &JNIStr = jni_str!("org/bukkit/command/CommandSender");

impl<'local> CommandSender<'local> {
    /// Verify the JObject is an `org.bukkit.command.CommandSender` and reinterpret as a borrowed
    /// `&CommandSender`.
    pub(crate) fn wrap_ref<'a>(
        env: &mut Env<'_>,
        obj: &'a JObject<'local>,
    ) -> jni::errors::Result<&'a Self> {
        let class = env.find_class(CLASS_NAME)?;
        if !env.is_instance_of(obj, &class)? {
            return Err(jni::errors::Error::WrongObjectType);
        }
        // SAFETY: just verified instanceof; CommandSender is repr(transparent) over JObject<'local>.
        Ok(unsafe { &*(obj as *const JObject<'local> as *const Self) })
    }

    pub fn name(&self, api: &mut Api) -> jni::errors::Result<String> {
        let env = api.jni();
        let name_obj = env
            .call_method(
                &self.obj,
                jni_str!("getName"),
                jni_sig!("()Ljava/lang/String;"),
                &[],
            )?
            .l()?;
        let name_jstr = env.cast_local::<JString>(name_obj)?;
        name_jstr.try_to_string(env)
    }

    pub fn send_message(&self, api: &mut Api, msg: impl AsRef<str>) -> jni::errors::Result<()> {
        let env = api.jni();
        let jstr = env.new_string(msg.as_ref())?;
        env.call_method(
            &self.obj,
            jni_str!("sendMessage"),
            jni_sig!("(Ljava/lang/String;)V"),
            &[JValue::Object(&jstr)],
        )?;
        Ok(())
    }
}

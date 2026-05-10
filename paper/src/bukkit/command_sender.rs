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

    /// Send a styled message.
    ///
    /// The input is parsed as [MiniMessage](https://docs.advntr.dev/minimessage/index.html), so
    /// `<green>hello, <yellow>world</yellow>` produces colored text. Plain text without tags
    /// renders unstyled.
    ///
    /// To bypass MiniMessage parsing (e.g., for user-controlled content where `<` shouldn't be
    /// interpreted as a tag), use [send_plain](Self::send_plain)
    pub fn send_message(&self, api: &mut Api, msg: impl AsRef<str>) -> jni::errors::Result<()> {
        let env = api.jni();
        let component = super::mini_message::deserialize(env, msg.as_ref())?;
        env.call_method(
            &self.obj,
            jni_str!("sendMessage"),
            jni_sig!("(Lnet/kyori/adventure/text/Component;)V"),
            &[JValue::Object(&component)],
        )?;
        Ok(())
    }

    /// Send a literal text message with no MiniMessage parsing or styling.
    ///
    /// Use for user-controlled content or anywhere `<` characters should be preserved as-is.
    pub fn send_plain(&self, api: &mut Api, msg: impl AsRef<str>) -> jni::errors::Result<()> {
        let env = api.jni();
        let jstr = env.new_string(msg.as_ref())?;
        let component = env
            .call_static_method(
                jni_str!("net/kyori/adventure/text/Component"),
                jni_str!("text"),
                jni_sig!("(Ljava/lang/String;)Lnet/kyori/adventure/text/TextComponent;"),
                &[JValue::Object(&jstr)],
            )?
            .l()?;
        env.call_method(
            &self.obj,
            jni_str!("sendMessage"),
            jni_sig!("(Lnet/kyori/adventure/text/Component;)V"),
            &[JValue::Object(&component)],
        )?;
        Ok(())
    }
}

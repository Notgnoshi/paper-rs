use jni::objects::{JObject, JString, JValue};
use jni::strings::JNIStr;
use jni::{Env, jni_sig, jni_str};

use crate::api::Api;

/// Type-erased wrapper for an `org.bukkit.command.CommandSender` JNI reference.
///
/// Provides the [`CommandSender`] trait surface (name, send_message, send_plain) plus narrowing
/// via [`CommandSenderInst::wrap_ref`].
///
/// `#[repr(transparent)]` over `JObject` so we can reinterpret a borrowed `&JObject` as a borrowed
/// `&CommandSenderInst` at dispatch time. The reinterpret is gated by an `is_instance_of` check in
/// `wrap_ref` so a Bukkit contract change can't silently feed us the wrong class.
#[repr(transparent)]
pub struct CommandSenderInst<'local> {
    obj: JObject<'local>,
}

const CLASS_NAME: &JNIStr = jni_str!("org/bukkit/command/CommandSender");

impl<'local> CommandSenderInst<'local> {
    /// Verify the JObject is an `org.bukkit.command.CommandSender` and reinterpret as a borrowed
    /// `&CommandSenderInst`.
    pub(crate) fn wrap_ref<'a>(
        env: &mut Env<'_>,
        obj: &'a JObject<'local>,
    ) -> jni::errors::Result<&'a Self> {
        let class = env.find_class(CLASS_NAME)?;
        if !env.is_instance_of(obj, &class)? {
            return Err(jni::errors::Error::WrongObjectType);
        }
        // SAFETY: just verified instanceof; CommandSenderInst is repr(transparent) over
        // JObject<'local>.
        Ok(unsafe { &*(obj as *const JObject<'local> as *const Self) })
    }
}

impl<'local> CommandSender<'local> for CommandSenderInst<'local> {
    fn as_jobject(&self) -> &JObject<'local> {
        &self.obj
    }
}

/// Rust trait mirror of Bukkit's `org.bukkit.command.CommandSender` interface.
///
/// Implementors provide [`CommandSender::as_jobject`]; methods like `name`, `send_message`, and
/// `send_plain` come for free via default impls that dispatch through it.
pub trait CommandSender<'local> {
    fn as_jobject(&self) -> &JObject<'local>;

    fn name(&self, api: &mut Api) -> jni::errors::Result<String> {
        let env = api.jni();
        let name_obj = env
            .call_method(
                self.as_jobject(),
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
    fn send_message(&self, api: &mut Api, msg: impl AsRef<str>) -> jni::errors::Result<()> {
        let env = api.jni();
        let component = super::mini_message::deserialize(env, msg.as_ref())?;
        env.call_method(
            self.as_jobject(),
            jni_str!("sendMessage"),
            jni_sig!("(Lnet/kyori/adventure/text/Component;)V"),
            &[JValue::Object(&component)],
        )?;
        Ok(())
    }

    /// Send a literal text message with no MiniMessage parsing or styling.
    ///
    /// Use for user-controlled content or anywhere `<` characters should be preserved as-is.
    fn send_plain(&self, api: &mut Api, msg: impl AsRef<str>) -> jni::errors::Result<()> {
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
            self.as_jobject(),
            jni_str!("sendMessage"),
            jni_sig!("(Lnet/kyori/adventure/text/Component;)V"),
            &[JValue::Object(&component)],
        )?;
        Ok(())
    }
}

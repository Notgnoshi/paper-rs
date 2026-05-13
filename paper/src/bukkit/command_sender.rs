use jni::objects::{JObject, JString, JValue};
use jni::{Env, jni_sig, jni_str};

use crate::api::Api;
use crate::ctx;
use crate::jobject_repr::JObjectRepr;

/// Type-erased wrapper for an `org.bukkit.command.CommandSender` JNI reference.
///
/// Provides the [`CommandSender`] trait surface (name, send_message, send_plain) plus
/// [`CommandSenderInst::cast`] to narrow to a specific subtype.
#[repr(transparent)]
pub struct CommandSenderInst<'local> {
    pub(crate) obj: JObject<'local>,
}

// SAFETY: `#[repr(transparent)]` over `JObject<'local>`
unsafe impl<'local> JObjectRepr<'local> for CommandSenderInst<'local> {}

impl<'local> CommandSenderInst<'local> {
    /// Verify the JObject is an `org.bukkit.command.CommandSender` and reinterpret as a borrowed
    /// `&CommandSenderInst`.
    pub(crate) fn wrap_ref<'a>(
        env: &mut Env<'_>,
        obj: &'a JObject<'local>,
    ) -> jni::errors::Result<&'a Self> {
        let class = ctx::cached_class(env, <Self as CommandSender>::CLASS_NAME)?;
        if !env.is_instance_of(obj, &class)? {
            return Err(jni::errors::Error::WrongObjectType);
        }
        Ok(Self::from_jobject_ref(obj))
    }

    /// Try to narrow this sender to a more specific subtype.
    ///
    /// Returns `None` if the sender is not a `T`.
    pub fn cast<T>(self, api: &mut Api) -> Option<T>
    where
        T: CommandSender<'local>,
    {
        let class = api.class(T::CLASS_NAME).ok()?;
        let env = api.jni();
        if env.is_instance_of(&self.obj, &class).ok()? {
            // SAFETY: just verified instanceof.
            Some(unsafe { T::from_obj(self.obj) })
        } else {
            None
        }
    }
}

impl<'local> CommandSender<'local> for CommandSenderInst<'local> {
    const CLASS_NAME: &'static str = "org/bukkit/command/CommandSender";

    unsafe fn from_obj(obj: JObject<'local>) -> Self {
        Self { obj }
    }

    fn as_jobject(&self) -> &JObject<'local> {
        &self.obj
    }
}

/// Rust trait mirror of Bukkit's `org.bukkit.command.CommandSender` interface.
///
/// Implementors provide the three required items below; methods like `name`, `send_message`, and
/// `send_plain` come for free via default impls that dispatch through [`as_jobject`].
///
/// `CLASS_NAME` and `from_obj` carry the narrowing infrastructure used by
/// [`CommandSenderInst::cast`].
pub trait CommandSender<'local>: Sized {
    /// Slash-delimited JVM class name, e.g. `"org/bukkit/command/CommandSender"`.
    const CLASS_NAME: &'static str;

    /// # SAFETY
    ///
    /// obj must be a JNI ref to a Java instance of `CLASS_NAME`.
    unsafe fn from_obj(obj: JObject<'local>) -> Self;

    fn as_jobject(&self) -> &JObject<'local>;

    fn name(&self, api: &mut Api) -> eyre::Result<String> {
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
        Ok(name_jstr.try_to_string(env)?)
    }

    /// Send a styled message.
    ///
    /// The input is parsed as [MiniMessage](https://docs.advntr.dev/minimessage/index.html), so
    /// `<green>hello, <yellow>world</yellow>` produces colored text. Plain text without tags
    /// renders unstyled.
    ///
    /// To bypass MiniMessage parsing (e.g., for user-controlled content where `<` shouldn't be
    /// interpreted as a tag), use [send_plain](Self::send_plain)
    fn send_message(&self, api: &mut Api, msg: impl AsRef<str>) -> eyre::Result<()> {
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
    fn send_plain(&self, api: &mut Api, msg: impl AsRef<str>) -> eyre::Result<()> {
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

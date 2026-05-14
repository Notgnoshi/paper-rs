use jni::objects::{JObject, JString, JValue};
use jni::{Env, jni_sig, jni_str};

use crate::api::Api;
use crate::ctx;
use crate::jobject_repr::{JClassCast, JObjectRepr};

/// Type-erased wrapper for an `org.bukkit.command.CommandSender` JNI reference.
#[repr(transparent)]
pub struct CommandSenderInst<'local> {
    pub(crate) obj: JObject<'local>,
}

unsafe impl<'local> JObjectRepr<'local> for CommandSenderInst<'local> {}
unsafe impl<'local> JClassCast<'local> for CommandSenderInst<'local> {
    const CLASS_NAME: &'static str = "org/bukkit/command/CommandSender";
}
impl<'local> CommandSender<'local> for CommandSenderInst<'local> {}

impl<'local> CommandSenderInst<'local> {
    pub(crate) fn wrap_ref<'a>(
        env: &mut Env<'_>,
        obj: &'a JObject<'local>,
    ) -> jni::errors::Result<&'a Self> {
        let class = ctx::cached_class(env, <Self as JClassCast>::CLASS_NAME)?;
        if !env.is_instance_of(obj, &class)? {
            return Err(jni::errors::Error::WrongObjectType);
        }
        Ok(Self::from_jobject_ref(obj))
    }

    pub fn cast<T>(self, api: &mut Api<'_, 'local>) -> Option<T>
    where
        T: JClassCast<'local> + CommandSender<'local>,
    {
        let class = api.class(T::CLASS_NAME).ok()?;
        let env = api.jni();
        if env.is_instance_of(&self.obj, &class).ok()? {
            Some(unsafe { T::from_jobject(self.obj) })
        } else {
            None
        }
    }
}

/// Rust trait mirror of Bukkit's `org.bukkit.command.CommandSender` interface.
pub trait CommandSender<'local>: JObjectRepr<'local> {
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

    /// Parsed as [MiniMessage](https://docs.advntr.dev/minimessage/index.html). For literal text,
    /// use [`send_plain`](Self::send_plain).
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

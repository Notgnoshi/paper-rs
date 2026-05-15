use jni::objects::{JString, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::jobject_repr::JObjectRepr;
use crate::papermc_jobject_inst;

papermc_jobject_inst! {
    pub CommandSenderInst<'local> = "org/bukkit/command/CommandSender": CommandSender;
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

use jni::jni_str;
use jni::objects::JObject;
use jni::strings::JNIStr;

use super::Entity;
use crate::bukkit::{Audience, CommandSender};

/// Wrapper for an `org.bukkit.entity.Player` JNI reference.
///
/// `Player` mirrors Bukkit's `Player` interface, which extends both `Entity` and `CommandSender`
/// (plus several intermediate interfaces not yet wrapped). This type impls both Rust traits, so
/// `Entity` methods (narrowing via `EntityInst::cast`) and `CommandSender` methods (`name`,
/// `send_message`, `send_plain`) are available on a `Player` value.
#[repr(transparent)]
pub struct Player<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> Entity<'local> for Player<'local> {
    const CLASS_NAME: &'static JNIStr = jni_str!("org/bukkit/entity/Player");

    unsafe fn from_obj(obj: JObject<'local>) -> Self {
        Self { obj }
    }
}

impl<'local> CommandSender<'local> for Player<'local> {
    const CLASS_NAME: &'static JNIStr = jni_str!("org/bukkit/entity/Player");

    unsafe fn from_obj(obj: JObject<'local>) -> Self {
        Self { obj }
    }

    fn as_jobject(&self) -> &JObject<'local> {
        &self.obj
    }
}

impl<'local> Audience<'local> for Player<'local> {
    fn as_jobject(&self) -> &JObject<'local> {
        &self.obj
    }
}

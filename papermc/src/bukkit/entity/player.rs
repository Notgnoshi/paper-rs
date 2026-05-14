use jni::objects::JObject;

use super::Entity;
use crate::bukkit::{Audience, CommandSender};
use crate::jobject_repr::{JClassCast, JObjectRepr};

/// Wrapper for an `org.bukkit.entity.Player` JNI reference.
#[repr(transparent)]
pub struct Player<'local> {
    pub(crate) obj: JObject<'local>,
}

unsafe impl<'local> JObjectRepr<'local> for Player<'local> {}
unsafe impl<'local> JClassCast<'local> for Player<'local> {
    const CLASS_NAME: &'static str = "org/bukkit/entity/Player";
}
impl<'local> Entity<'local> for Player<'local> {}
impl<'local> CommandSender<'local> for Player<'local> {}
impl<'local> Audience<'local> for Player<'local> {}

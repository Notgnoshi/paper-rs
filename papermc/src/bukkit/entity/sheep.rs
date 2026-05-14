use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use super::Entity;
use crate::api::Api;
use crate::bukkit::DyeColor;
use crate::jobject_repr::{JClassCast, JObjectRepr};

/// Wrapper for an `org.bukkit.entity.Sheep` JNI reference.
#[repr(transparent)]
pub struct Sheep<'local> {
    obj: JObject<'local>,
}

unsafe impl<'local> JObjectRepr<'local> for Sheep<'local> {}
unsafe impl<'local> JClassCast<'local> for Sheep<'local> {
    const CLASS_NAME: &'static str = "org/bukkit/entity/Sheep";
}
impl<'local> Entity<'local> for Sheep<'local> {}

impl<'local> Sheep<'local> {
    pub fn set_color(&mut self, api: &mut Api, color: DyeColor) -> eyre::Result<()> {
        let env = api.jni();
        let dye = color.as_java(env)?;
        env.call_method(
            &self.obj,
            jni_str!("setColor"),
            jni_sig!("(Lorg/bukkit/DyeColor;)V"),
            &[JValue::Object(&dye)],
        )?;
        Ok(())
    }
}

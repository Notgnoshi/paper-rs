use jni::objects::{JObject, JValue};
use jni::strings::JNIStr;
use jni::{jni_sig, jni_str};

use super::Entity;
use crate::api::Api;
use crate::bukkit::DyeColor;

/// Wrapper for an `org.bukkit.entity.Sheep` JNI reference.
pub struct Sheep<'local> {
    obj: JObject<'local>,
}

impl<'local> Entity<'local> for Sheep<'local> {
    const CLASS_NAME: &'static JNIStr = jni_str!("org/bukkit/entity/Sheep");

    unsafe fn from_obj(obj: JObject<'local>) -> Self {
        Self { obj }
    }
}

impl<'local> Sheep<'local> {
    pub fn set_color(&mut self, api: &mut Api, color: DyeColor) -> jni::errors::Result<()> {
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

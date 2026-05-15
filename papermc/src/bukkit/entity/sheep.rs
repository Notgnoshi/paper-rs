use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use super::Entity;
use crate::api::Api;
use crate::bukkit::DyeColor;
use crate::papermc_jobject;

papermc_jobject! {
    pub Sheep<'local> = "org/bukkit/entity/Sheep": Entity;
}

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

use jni::objects::JObject;
use jni::strings::JNIStr;
use jni::{Env, jni_sig, jni_str};

use super::Event;
use crate::api::Api;
use crate::bukkit::EntityInst;

/// Marker type. Used in `PluginBuilder::on::<PlayerInteractEntityEvent>`.
pub struct PlayerInteractEntityEvent;

/// Wrapper for an `org.bukkit.event.player.PlayerInteractEntityEvent` JNI reference.
///
/// Plugin handlers receive this by `&` in their handler bodies.
///
/// `#[repr(transparent)]` so dispatch can reinterpret a borrowed `&JObject` as a borrowed
/// `&PlayerInteractEntityEventRef`.
#[repr(transparent)]
pub struct PlayerInteractEntityEventRef<'local> {
    obj: JObject<'local>,
}

impl Event for PlayerInteractEntityEvent {
    type Wrapper<'local> = PlayerInteractEntityEventRef<'local>;
    const CLASS_NAME: &'static JNIStr =
        jni_str!("org/bukkit/event/player/PlayerInteractEntityEvent");

    fn wrap<'a, 'local>(
        env: &mut Env<'_>,
        obj: &'a JObject<'local>,
    ) -> jni::errors::Result<&'a Self::Wrapper<'local>> {
        let class = env.find_class(Self::CLASS_NAME)?;
        if !env.is_instance_of(obj, &class)? {
            return Err(jni::errors::Error::WrongObjectType);
        }
        // SAFETY: just verified instanceof; PlayerInteractEntityEventRef is repr(transparent) over
        // JObject<'local>.
        Ok(unsafe {
            &*(obj as *const JObject<'local> as *const PlayerInteractEntityEventRef<'local>)
        })
    }
}

impl<'local> PlayerInteractEntityEventRef<'local> {
    pub fn right_clicked(
        &self,
        api: &mut Api<'_, 'local>,
    ) -> eyre::Result<EntityInst<'local>> {
        let env = api.jni();
        let entity = env
            .call_method(
                &self.obj,
                jni_str!("getRightClicked"),
                jni_sig!("()Lorg/bukkit/entity/Entity;"),
                &[],
            )?
            .l()?;
        Ok(EntityInst::new(entity))
    }
}

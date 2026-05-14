use jni::objects::JObject;
use jni::{jni_sig, jni_str};

use super::Event;
use crate::api::Api;
use crate::bukkit::EntityInst;
use crate::jobject_repr::JObjectRepr;

/// Marker type. Used in `SetupApi::register_event`.
pub struct PlayerInteractEntityEvent;

/// Wrapper for an `org.bukkit.event.player.PlayerInteractEntityEvent` JNI reference.
///
/// Plugin handlers receive this by `&` in their handler bodies.
#[repr(transparent)]
pub struct PlayerInteractEntityEventRef<'local> {
    obj: JObject<'local>,
}

// SAFETY: `#[repr(transparent)]` over `JObject<'local>`
unsafe impl<'local> JObjectRepr<'local> for PlayerInteractEntityEventRef<'local> {}

impl Event for PlayerInteractEntityEvent {
    type Wrapper<'local> = PlayerInteractEntityEventRef<'local>;
    const CLASS_NAME: &'static str = "org/bukkit/event/player/PlayerInteractEntityEvent";
}

impl<'local> PlayerInteractEntityEventRef<'local> {
    pub fn right_clicked(&self, api: &mut Api<'_, 'local>) -> eyre::Result<EntityInst<'local>> {
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

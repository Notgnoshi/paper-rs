use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::EntityInst;
use crate::papermc_event;

papermc_event! {
    pub PlayerInteractEntityEvent => PlayerInteractEntityEventRef
        = "org/bukkit/event/player/PlayerInteractEntityEvent";
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

use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::{EntityInst, Player};
use crate::papermc_event;

papermc_event! {
    pub EntityDamageByEntityEvent => EntityDamageByEntityEventRef
        = "org/bukkit/event/entity/EntityDamageByEntityEvent";
}

impl<'local> EntityDamageByEntityEventRef<'local> {
    /// The entity being damaged.
    pub fn entity(&self, api: &mut Api<'_, 'local>) -> eyre::Result<EntityInst<'local>> {
        let env = api.jni();
        let entity = env
            .call_method(
                &self.obj,
                jni_str!("getEntity"),
                jni_sig!("()Lorg/bukkit/entity/Entity;"),
                &[],
            )?
            .l()?;
        Ok(EntityInst::new(entity))
    }

    /// The entity that dealt the damage. For projectile damage this is the projectile itself;
    /// see [`player_attacker`](Self::player_attacker) for the shooter.
    pub fn damager(&self, api: &mut Api<'_, 'local>) -> eyre::Result<EntityInst<'local>> {
        let env = api.jni();
        let entity = env
            .call_method(
                &self.obj,
                jni_str!("getDamager"),
                jni_sig!("()Lorg/bukkit/entity/Entity;"),
                &[],
            )?
            .l()?;
        Ok(EntityInst::new(entity))
    }

    /// Walks one level of `Projectile.getShooter()` so projectile damage attributes to the player.
    pub fn player_attacker(
        &self,
        api: &mut Api<'_, 'local>,
    ) -> eyre::Result<Option<Player<'local>>> {
        let damager = self.damager(api)?;
        let player_class = api.class("org/bukkit/entity/Player")?;
        let env = api.jni();

        if !damager.obj.is_null() && env.is_instance_of(&damager.obj, &player_class)? {
            return Ok(Some(unsafe { Player::from_jobject(damager.obj) }));
        }

        let projectile_class = api.class("org/bukkit/entity/Projectile")?;
        let env = api.jni();
        if !damager.obj.is_null() && env.is_instance_of(&damager.obj, &projectile_class)? {
            let shooter_obj = env
                .call_method(
                    &damager.obj,
                    jni_str!("getShooter"),
                    jni_sig!("()Lorg/bukkit/projectiles/ProjectileSource;"),
                    &[],
                )?
                .l()?;
            // JNI's IsInstanceOf returns TRUE for null.
            if !shooter_obj.is_null() && env.is_instance_of(&shooter_obj, &player_class)? {
                return Ok(Some(unsafe { Player::from_jobject(shooter_obj) }));
            }
        }

        Ok(None)
    }
}

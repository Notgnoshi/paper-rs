use jni::objects::JObject;
use jni::{jni_sig, jni_str};

use super::Event;
use crate::api::Api;
use crate::bukkit::{Entity, EntityInst, Player};
use crate::jobject_repr::JObjectRepr;

/// Marker type. Used in `SetupApi::register_event`
pub struct EntityDamageByEntityEvent;

/// Wrapper for an `org.bukkit.event.entity.EntityDamageByEntityEvent` JNI reference.
#[repr(transparent)]
pub struct EntityDamageByEntityEventRef<'local> {
    obj: JObject<'local>,
}

// SAFETY: `#[repr(transparent)]` over `JObject<'local>`
unsafe impl<'local> JObjectRepr<'local> for EntityDamageByEntityEventRef<'local> {}

impl Event for EntityDamageByEntityEvent {
    type Wrapper<'local> = EntityDamageByEntityEventRef<'local>;
    const CLASS_NAME: &'static str = "org/bukkit/event/entity/EntityDamageByEntityEvent";
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

    /// The entity that dealt the damage.
    ///
    /// For projectile damage this is the projectile itself (e.g., an arrow); to find the
    /// player who ultimately caused the damage, use [`player_attacker`](Self::player_attacker).
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

    /// If the damage was ultimately caused by a player (directly, or via a player-shot
    /// projectile / thrown potion), returns that player. Otherwise `None`.
    ///
    /// `damager` itself returns the immediate damaging entity, which for projectiles is the
    /// projectile, not the shooter. This helper walks one level of indirection through
    /// `Projectile.getShooter()` so callers don't have to.
    pub fn player_attacker(
        &self,
        api: &mut Api<'_, 'local>,
    ) -> eyre::Result<Option<Player<'local>>> {
        let damager = self.damager(api)?;
        let player_class = api.class("org/bukkit/entity/Player")?;
        let env = api.jni();

        // Direct player damage: punch, sword, etc.
        if !damager.obj.is_null() && env.is_instance_of(&damager.obj, &player_class)? {
            // SAFETY: verified instanceof Player.
            return Ok(Some(unsafe { <Player as Entity>::from_obj(damager.obj) }));
        }

        // Projectile damage: arrow, thrown potion, trident, etc. Check the shooter.
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
            // JNI's IsInstanceOf returns TRUE for null, so null-check first.
            if !shooter_obj.is_null() && env.is_instance_of(&shooter_obj, &player_class)? {
                // SAFETY: verified non-null and instanceof Player.
                return Ok(Some(unsafe { <Player as Entity>::from_obj(shooter_obj) }));
            }
        }

        Ok(None)
    }
}

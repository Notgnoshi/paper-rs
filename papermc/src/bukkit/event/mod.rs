use jni::Env;
use jni::objects::JObject;

use crate::ctx;
use crate::jobject_repr::JObjectRepr;

mod entity_damage_by_entity_event;
mod player_interact_entity_event;

pub use entity_damage_by_entity_event::{EntityDamageByEntityEvent, EntityDamageByEntityEventRef};
pub use player_interact_entity_event::{PlayerInteractEntityEvent, PlayerInteractEntityEventRef};

/// Trait implemented by event marker types.
///
/// The marker (e.g. `PlayerInteractEntityEvent`) is a ZST without a lifetime; the associated
/// `Wrapper<'local>` is the lifetime'd typed reference plugin authors receive in handler bodies.
///
/// This indirection sidesteps Rust's lack of HKT: we want `SetupApi::register_event` to accept any
/// event marker and dispatch to a handler whose argument is the corresponding wrapper at the
/// dispatch-time JNI lifetime.
pub trait Event: 'static {
    type Wrapper<'local>: JObjectRepr<'local>;
    /// Slash-delimited JVM class name, e.g. `"org/bukkit/event/player/PlayerInteractEntityEvent"`.
    const CLASS_NAME: &'static str;

    /// Verify `obj` is an instance of `CLASS_NAME` and reinterpret as `&Wrapper`. Returns
    /// `Err(WrongObjectType)` if the check fails.
    ///
    /// The default impl is appropriate for every event whose `Wrapper` is a `#[repr(transparent)]`
    /// newtype over `JObject<'local>` (which the [`JObjectRepr`] bound already requires); there's
    /// no reason to override it.
    fn wrap<'a, 'local>(
        env: &mut Env<'_>,
        obj: &'a JObject<'local>,
    ) -> jni::errors::Result<&'a Self::Wrapper<'local>> {
        let class = ctx::cached_class(env, Self::CLASS_NAME)?;
        if !env.is_instance_of(obj, &class)? {
            return Err(jni::errors::Error::WrongObjectType);
        }
        Ok(Self::Wrapper::from_jobject_ref(obj))
    }
}

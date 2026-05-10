use jni::Env;
use jni::objects::JObject;
use jni::strings::JNIStr;

mod player_interact_entity_event;

pub use player_interact_entity_event::{PlayerInteractEntityEvent, PlayerInteractEntityEventRef};

/// Trait implemented by event marker types.
///
/// The marker (e.g. `PlayerInteractEntityEvent`) is a ZST without a lifetime; the associated
/// `Wrapper<'local>` is the lifetime'd typed reference plugin authors receive in handler bodies.
///
/// This indirection sidesteps Rust's lack of HKT: we want `PluginBuilder::on` to accept any event
/// marker and dispatch to a handler whose argument is the corresponding wrapper at the
/// dispatch-time JNI lifetime.
///
/// The `wrap` method is verified at the dispatch boundary: it does an `is_instance_of` check before
/// reinterpreting the JObject as a `Wrapper`, so a Bukkit contract change can't silently feed us
/// the wrong class.
pub trait Event: 'static {
    type Wrapper<'local>;
    const CLASS_NAME: &'static JNIStr;
    /// Verify `obj` is an instance of `CLASS_NAME` and reinterpret as `&Wrapper`. Returns
    /// `Err(WrongObjectType)` if the check fails.
    ///
    /// SAFETY contract for implementors: `Wrapper<'local>` MUST be `#[repr(transparent)]` over
    /// `JObject<'local>`. paper-rs's built-in impls satisfy this; user-defined impls must too.
    fn wrap<'a, 'local>(
        env: &mut Env<'_>,
        obj: &'a JObject<'local>,
    ) -> jni::errors::Result<&'a Self::Wrapper<'local>>;
}

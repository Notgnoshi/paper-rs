use jni::objects::JObject;

use crate::api::Api;

mod player;
mod sheep;

pub use player::Player;
pub use sheep::Sheep;

/// Type-erased wrapper for an `org.bukkit.entity.Entity` JNI reference.
///
/// Use [`EntityInst::cast`] to narrow to a specific subtype like [`Sheep`].
///
/// `#[repr(transparent)]` so papermc can reinterpret a borrowed `&JObject` as a borrowed
/// `&EntityInst` at dispatch time.
#[repr(transparent)]
pub struct EntityInst<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> EntityInst<'local> {
    pub(crate) fn new(obj: JObject<'local>) -> Self {
        Self { obj }
    }

    /// Try to narrow this entity to a more specific subtype.
    ///
    /// Returns `None` if the entity is not a `T`.
    ///
    /// ```ignore
    /// if let Some(mut sheep) = entity.cast::<Sheep>(api) {
    ///     // ...
    /// }
    /// ```
    pub fn cast<T>(self, api: &mut Api) -> Option<T>
    where
        T: Entity<'local>,
    {
        let class = api.class(T::CLASS_NAME).ok()?;
        let env = api.jni();
        if env.is_instance_of(&self.obj, &class).ok()? {
            // SAFETY: just verified instanceof.
            Some(unsafe { T::from_obj(self.obj) })
        } else {
            None
        }
    }
}

/// Rust trait mirror of Bukkit's `org.bukkit.entity.Entity` interface.
///
/// Currently carries the narrowing infrastructure (`CLASS_NAME`, `from_obj`) used by
/// [`EntityInst::cast`]. Interface methods will be added as default impls when callers need them.
pub trait Entity<'local>: Sized {
    /// Slash-delimited JVM class name, e.g. `"org/bukkit/entity/Sheep"`.
    const CLASS_NAME: &'static str;
    /// # SAFETY
    ///
    /// obj must be a JNI ref to a Java instance of `CLASS_NAME`.
    unsafe fn from_obj(obj: JObject<'local>) -> Self;
}

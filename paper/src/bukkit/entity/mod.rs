use jni::objects::JObject;
use jni::strings::JNIStr;

use crate::api::Api;

mod sheep;

pub use sheep::Sheep;

/// Wrapper for an `org.bukkit.entity.Entity` JNI reference.
///
/// `#[repr(transparent)]` so paper-rs can reinterpret a borrowed `&JObject` as a borrowed `&Entity`
/// at dispatch time.
#[repr(transparent)]
pub struct Entity<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> Entity<'local> {
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
        T: IsEntity<'local>,
    {
        let env = api.jni();
        let class = env.find_class(T::CLASS_NAME).ok()?;
        if env.is_instance_of(&self.obj, &class).ok()? {
            // SAFETY: just verified instanceof.
            Some(unsafe { T::from_obj(self.obj) })
        } else {
            None
        }
    }
}

/// Marker trait for entity subtypes that paper-rs has typed wrappers for.
pub trait IsEntity<'local>: Sized {
    const CLASS_NAME: &'static JNIStr;
    /// # SAFETY
    ///
    /// obj must be a JNI ref to a Java instance of `CLASS_NAME`.
    unsafe fn from_obj(obj: JObject<'local>) -> Self;
}

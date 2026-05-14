use crate::api::Api;
use crate::jobject_repr::{JClassCast, JObjectRepr};

mod player;
mod sheep;

use jni::objects::JObject;
pub use player::Player;
pub use sheep::Sheep;

/// Type-erased wrapper for an `org.bukkit.entity.Entity` JNI reference. Use [`EntityInst::cast`]
/// to narrow to a specific subtype like [`Sheep`].
#[repr(transparent)]
pub struct EntityInst<'local> {
    pub(crate) obj: JObject<'local>,
}

unsafe impl<'local> JObjectRepr<'local> for EntityInst<'local> {}
unsafe impl<'local> JClassCast<'local> for EntityInst<'local> {
    const CLASS_NAME: &'static str = "org/bukkit/entity/Entity";
}
impl<'local> Entity<'local> for EntityInst<'local> {}

impl<'local> EntityInst<'local> {
    pub(crate) fn new(obj: JObject<'local>) -> Self {
        Self { obj }
    }

    pub fn cast<T>(self, api: &mut Api<'_, 'local>) -> Option<T>
    where
        T: JClassCast<'local> + Entity<'local>,
    {
        let class = api.class(T::CLASS_NAME).ok()?;
        let env = api.jni();
        if env.is_instance_of(&self.obj, &class).ok()? {
            Some(unsafe { T::from_jobject(self.obj) })
        } else {
            None
        }
    }
}

/// Rust trait mirror of Bukkit's `org.bukkit.entity.Entity` interface.
pub trait Entity<'local>: JObjectRepr<'local> {}

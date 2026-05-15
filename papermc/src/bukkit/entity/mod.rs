use crate::jobject_repr::JObjectRepr;
use crate::papermc_jobject_inst;

mod player;
mod sheep;

pub use player::Player;
pub use sheep::Sheep;

papermc_jobject_inst! {
    pub EntityInst<'local> = "org/bukkit/entity/Entity": Entity;
}

/// Rust trait mirror of Bukkit's `org.bukkit.entity.Entity` interface.
pub trait Entity<'local>: JObjectRepr<'local> {}

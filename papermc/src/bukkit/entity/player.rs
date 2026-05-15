use super::Entity;
use crate::bukkit::{Audience, CommandSender};
use crate::papermc_jobject;

papermc_jobject! {
    pub Player<'local> = "org/bukkit/entity/Player": Entity, CommandSender, Audience;
}

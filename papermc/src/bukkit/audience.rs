use jni::objects::JValue;
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::dialog::Dialog;
use crate::jobject_repr::JObjectRepr;

/// Rust trait mirror of Adventure's `net.kyori.adventure.audience.Audience` interface.
pub trait Audience<'local>: JObjectRepr<'local> {
    fn show_dialog(&self, api: &mut Api<'_, 'local>, dialog: &Dialog<'local>) -> eyre::Result<()> {
        let env = api.jni();
        env.call_method(
            self.as_jobject(),
            jni_str!("showDialog"),
            jni_sig!("(Lnet/kyori/adventure/dialog/DialogLike;)V"),
            &[JValue::Object(&dialog.obj)],
        )?;
        Ok(())
    }
}

use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::bukkit::dialog::Dialog;

/// Rust trait mirror of Adventure's `net.kyori.adventure.audience.Audience` interface.
///
/// Implementors provide [`Audience::as_jobject`]; methods like [`Audience::show_dialog`] come
/// for free via default impls that dispatch through it.
///
/// Adventure's full Audience has many methods (sendMessage, playSound, sendActionBar, etc.).
/// Only the surface needed by current callers is wrapped; extend the trait as callers arrive.
pub trait Audience<'local> {
    fn as_jobject(&self) -> &JObject<'local>;

    /// Show a dialog to this audience.
    ///
    /// The Java signature is `Audience.showDialog(DialogLike)`; `Dialog` extends `DialogLike`.
    fn show_dialog(
        &self,
        api: &mut Api<'_, 'local>,
        dialog: &Dialog<'local>,
    ) -> jni::errors::Result<()> {
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

use jni::objects::JObject;
use jni::strings::JNIStr;
use jni::{Env, jni_sig, jni_str};

/// Mirror of `io.papermc.paper.registry.data.dialog.DialogBase.DialogAfterAction`.
///
/// Controls what happens after the player interacts with the dialog.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DialogAfterAction {
    /// Close the dialog and return to the previous non-dialog screen (if any).
    Close,
    /// Do nothing; keep the current screen open.
    None,
    /// Replace the dialog with a "waiting for response" screen.
    WaitForResponse,
}

impl DialogAfterAction {
    pub(crate) fn as_java<'local>(
        &self,
        env: &mut Env<'local>,
    ) -> jni::errors::Result<JObject<'local>> {
        let field: &JNIStr = match self {
            Self::Close => jni_str!("CLOSE"),
            Self::None => jni_str!("NONE"),
            Self::WaitForResponse => jni_str!("WAIT_FOR_RESPONSE"),
        };
        env.get_static_field(
            jni_str!("io/papermc/paper/registry/data/dialog/DialogBase$DialogAfterAction"),
            field,
            jni_sig!("Lio/papermc/paper/registry/data/dialog/DialogBase$DialogAfterAction;"),
        )?
        .l()
    }
}

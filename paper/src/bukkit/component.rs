use jni::objects::JObject;

use crate::api::Api;

/// Wrapper for a `net.kyori.adventure.text.Component` JNI reference.
///
/// This is a minimal handle; no fluent builder yet. Construct from a MiniMessage string with
/// [`Component::mini_message`]. A typed Component builder is a separate effort.
#[repr(transparent)]
pub struct Component<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> Component<'local> {
    /// Parse the given MiniMessage string into a Component.
    ///
    /// See <https://docs.advntr.dev/minimessage/index.html> for tag syntax.
    pub fn mini_message(api: &mut Api<'_, 'local>, text: &str) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = super::mini_message::deserialize(env, text)?;
        Ok(Self { obj })
    }
}

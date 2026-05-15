use std::time::Duration;

use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::papermc_builder;

/// Wrapper for `net.kyori.adventure.text.event.ClickCallback$Options`.
#[repr(transparent)]
pub struct ClickCallbackOptions<'local> {
    pub(crate) obj: JObject<'local>,
}

impl<'local> ClickCallbackOptions<'local> {
    pub fn builder(api: &mut Api<'_, 'local>) -> eyre::Result<ClickCallbackOptionsBuilder<'local>> {
        let env = api.jni();
        let obj = env
            .call_static_method(
                jni_str!("net/kyori/adventure/text/event/ClickCallback$Options"),
                jni_str!("builder"),
                jni_sig!("()Lnet/kyori/adventure/text/event/ClickCallback$Options$Builder;"),
                &[],
            )?
            .l()?;
        Ok(ClickCallbackOptionsBuilder { obj })
    }
}

papermc_builder! {
    pub ClickCallbackOptionsBuilder<'local> -> ClickCallbackOptions<'local>
        builds "()Lnet/kyori/adventure/text/event/ClickCallback$Options;";
}

impl<'local> ClickCallbackOptionsBuilder<'local> {
    pub fn uses(self, api: &mut Api<'_, 'local>, count: i32) -> eyre::Result<Self> {
        let env = api.jni();
        env.call_method(
            &self.obj,
            jni_str!("uses"),
            jni_sig!("(I)Lnet/kyori/adventure/text/event/ClickCallback$Options$Builder;"),
            &[JValue::Int(count)],
        )?;
        Ok(self)
    }

    /// Converts the Rust Duration to a Java `Duration`, which implements `TemporalAmount`.
    pub fn lifetime(self, api: &mut Api<'_, 'local>, duration: Duration) -> eyre::Result<Self> {
        let env = api.jni();
        let java_duration = env
            .call_static_method(
                jni_str!("java/time/Duration"),
                jni_str!("ofMillis"),
                jni_sig!("(J)Ljava/time/Duration;"),
                &[JValue::Long(duration.as_millis() as i64)],
            )?
            .l()?;
        env.call_method(
            &self.obj,
            jni_str!("lifetime"),
            jni_sig!(
                "(Ljava/time/temporal/TemporalAmount;)Lnet/kyori/adventure/text/event/ClickCallback$Options$Builder;"
            ),
            &[JValue::Object(&java_duration)],
        )?;
        Ok(self)
    }
}

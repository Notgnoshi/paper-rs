use crate::api::Api;
use crate::setup_api::SetupApi;

/// User-facing plugin trait that corresponds to a Paper Plugin
///
/// [Plugin::on_enable] constructs the plugin instance and registers handlers via [SetupApi]. The
/// returned value is held by papermc for the duration of the load and dropped after `on_disable`
/// returns.
///
/// [Plugin::on_disable] is invoked before papermc-loader dlcloses the plugin .so. The default impl
/// is a no-op; override to release explicit resources beyond what `Drop` handles.
pub trait Plugin: Sized + Send + 'static {
    fn on_enable(api: &mut SetupApi<'_, '_, Self>) -> eyre::Result<Self>;

    fn on_disable(&mut self, _api: &mut Api<'_, '_>) -> eyre::Result<()> {
        Ok(())
    }
}

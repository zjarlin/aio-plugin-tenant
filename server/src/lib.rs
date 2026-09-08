mod routes;
mod service;

pub use service::TenantService;

use anyhow::{Context as _, Result};
use axum::Router;
use dill::CatalogBuilder;

pub fn register(builder: &mut CatalogBuilder) -> Result<()> {
    builder.add_value(TenantService::from_env()?);
    Ok(())
}

pub fn router(catalog: &dill::Catalog) -> Result<Router> {
    let tenants = catalog
        .get_one::<TenantService>()
        .context("租户服务未注册")?;
    let identity = aio_plugin_identity_server::service(catalog)?;
    Ok(routes::router(tenants, identity))
}

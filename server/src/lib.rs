use axum::{Router, routing::get};
use dill::CatalogBuilder;

#[derive(Debug)]
pub struct TenantService;

pub fn register(builder: &mut CatalogBuilder) {
    builder.add_value(TenantService);
}

pub fn router(_catalog: &dill::Catalog) -> anyhow::Result<Router> {
    Ok(Router::new().route("/api/plugins/tenant/health", get(|| async { "ok" })))
}

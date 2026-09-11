use aio_plugin_tenant_model::{
    CreateTenantRequest, SwitchTenantRequest, TenantErrorResponse, TenantItem, TenantResponse,
};
use gloo_net::http::{Request, Response};
use serde::Deserialize;

#[derive(Deserialize)]
struct SessionAccess {
    permissions: Vec<String>,
}

pub(super) async fn load() -> Result<(Vec<TenantItem>, bool), String> {
    let tenants = get::<Vec<TenantItem>>("/api/tenants").await?;
    let session = get::<Option<SessionAccess>>("/api/auth/session").await?;
    Ok((
        tenants,
        session.is_some_and(|s| s.permissions.iter().any(|p| p == "tenant:manage")),
    ))
}

pub(super) async fn save(id: Option<String>, label: String) -> Result<(), String> {
    let request = match id {
        Some(id) => Request::put(&format!("/api/tenants/{id}")),
        None => Request::post("/api/tenants"),
    };
    let response = request
        .json(&CreateTenantRequest { label })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    check(response).await
}

pub(super) async fn switch(id: String) -> Result<(), String> {
    let response = Request::post("/api/tenants/switch")
        .json(&SwitchTenantRequest { tenant_id: id })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    check(response).await?;
    dioxus::document::eval("window.dispatchEvent(new Event('aio:catalog-invalidated')); return true;")
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

async fn get<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let response = Request::get(path).send().await.map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(error(response).await);
    }
    response
        .json::<TenantResponse<T>>()
        .await
        .map(|r| r.data)
        .map_err(|e| e.to_string())
}

async fn check(response: Response) -> Result<(), String> {
    if response.ok() {
        Ok(())
    } else {
        Err(error(response).await)
    }
}
async fn error(response: Response) -> String {
    let body = response.text().await.unwrap_or_default();
    serde_json::from_str::<TenantErrorResponse>(&body)
        .map(|r| r.error)
        .unwrap_or(body)
}

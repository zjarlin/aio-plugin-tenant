use std::sync::Arc;

use aio_plugin_identity_server::{IdentityService, SessionContext};
use aio_plugin_tenant_model::{
    CreateTenantRequest, SwitchTenantRequest, TenantErrorResponse, TenantItem, TenantResponse,
};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::TenantService;

#[derive(Clone)]
struct TenantState {
    tenants: Arc<TenantService>,
    identity: Arc<IdentityService>,
}

pub fn router(tenants: Arc<TenantService>, identity: Arc<IdentityService>) -> Router {
    Router::new()
        .route("/api/plugins/tenant/health", get(health))
        .route("/api/tenants", get(list).post(create))
        .route("/api/tenants/switch", post(switch))
        .with_state(TenantState { tenants, identity })
}

async fn health() -> &'static str {
    "ok"
}

async fn list(
    State(state): State<TenantState>,
    headers: HeaderMap,
) -> Result<Json<TenantResponse<Vec<TenantItem>>>, TenantHttpError> {
    let session = authenticate(&state, &headers).await?;
    Ok(Json(TenantResponse {
        data: state
            .tenants
            .list(&session.user_id, &session.tenant_id)
            .await?,
    }))
}

async fn create(
    State(state): State<TenantState>,
    headers: HeaderMap,
    Json(request): Json<CreateTenantRequest>,
) -> Result<Json<TenantResponse<TenantItem>>, TenantHttpError> {
    let session = authenticate(&state, &headers).await?;
    require(&session, "tenant:manage")?;
    Ok(Json(TenantResponse {
        data: state
            .tenants
            .create(&session.user_id, &request.label)
            .await?,
    }))
}

async fn switch(
    State(state): State<TenantState>,
    headers: HeaderMap,
    Json(request): Json<SwitchTenantRequest>,
) -> Result<StatusCode, TenantHttpError> {
    let session = authenticate(&state, &headers).await?;
    if !state
        .identity
        .switch_tenant(&session, &request.tenant_id)
        .await?
    {
        return Err(TenantHttpError::forbidden("当前账号不属于目标租户"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn authenticate(
    state: &TenantState,
    headers: &HeaderMap,
) -> Result<SessionContext, TenantHttpError> {
    state
        .identity
        .authenticate(headers)
        .await?
        .ok_or_else(|| TenantHttpError::unauthorized("会话无效或已过期"))
}

fn require(session: &SessionContext, permission: &str) -> Result<(), TenantHttpError> {
    if session
        .permissions
        .iter()
        .any(|candidate| candidate == permission)
    {
        Ok(())
    } else {
        Err(TenantHttpError::forbidden("当前角色没有租户管理权限"))
    }
}

struct TenantHttpError {
    status: StatusCode,
    message: String,
}

impl TenantHttpError {
    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }
}

impl<E> From<E> for TenantHttpError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: format!("{:#}", value.into()),
        }
    }
}

impl IntoResponse for TenantHttpError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(TenantErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

use aio_plugin_tenant_model::{
    CreateTenantRequest, SwitchTenantRequest, TenantErrorResponse, TenantItem, TenantResponse,
};
use az_dioxus_admin_shell::{
    ApplicationAccountItem, ApplicationAccountPlugin, ApplicationPage, ApplicationPlugin,
    ApplicationScene,
};
use az_ui_components::{
    badge::{Badge, BadgeVariant},
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogTitle},
    input::Input,
};
use dill::CatalogBuilder;
use dioxus::prelude::*;

#[derive(Debug)]
pub struct TenantPlugin;

impl ApplicationPlugin for TenantPlugin {
    fn pages(&self) -> Vec<ApplicationPage> {
        vec![ApplicationPage {
            id: "tenants",
            label: "租户管理",
            icon: Some("tenant"),
            scene: ApplicationScene {
                id: "system",
                label: "系统",
            },
            menu_path: Vec::new(),
            required_permission: None,
            render: TenantPage,
        }]
    }
}

impl ApplicationAccountPlugin for TenantPlugin {
    fn items(&self) -> Vec<ApplicationAccountItem> {
        vec![ApplicationAccountItem {
            id: "tenant-switcher".to_owned(),
            label: "切换租户".to_owned(),
            icon: Some("tenant".to_owned()),
            page_id: Some("tenants".to_owned()),
            required_permission: None,
            destructive: false,
        }]
    }
}

pub fn register(builder: &mut CatalogBuilder) {
    builder
        .add_value(TenantPlugin)
        .bind::<dyn ApplicationPlugin, TenantPlugin>()
        .bind::<dyn ApplicationAccountPlugin, TenantPlugin>();
}

#[allow(non_snake_case)]
fn TenantPage() -> Element {
    let tenants = use_resource(load_tenants);
    let mut creating = use_signal(|| false);
    let Some(result) = tenants.read().as_ref().cloned() else {
        return rsx! { p { "正在读取租户" } };
    };
    let tenants = match result {
        Ok(value) => value,
        Err(error) => return rsx! { p { role: "alert", "加载租户失败：{error}" } },
    };
    rsx! {
        section {
            div { class: "flex items-center justify-between gap-3",
                div {
                    h2 { "租户管理" }
                    p { "每个租户拥有独立的插件组合与锁定版本。" }
                }
                Button {
                    r#type: "button",
                    variant: ButtonVariant::Outline,
                    onclick: move |_| creating.set(true),
                    "新建租户"
                }
            }
            div { class: "grid gap-3 md:grid-cols-2",
                for tenant in tenants {
                    article { class: "border p-4",
                        div { class: "flex items-center justify-between gap-3",
                            h3 { "{tenant.label}" }
                            if tenant.current {
                                Badge { variant: BadgeVariant::Outline, "当前租户" }
                            } else {
                                Button {
                                    r#type: "button",
                                    variant: ButtonVariant::Ghost,
                                    onclick: move |_| {
                                        let tenant_id = tenant.id.clone();
                                        spawn(async move {
                                            if switch_tenant(tenant_id).await.is_ok() {
                                                reload();
                                            }
                                        });
                                    },
                                    "切换"
                                }
                            }
                        }
                    }
                }
            }
        }
        if creating() {
            CreateTenantDialog { on_close: move |_| creating.set(false) }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn CreateTenantDialog(on_close: EventHandler<()>) -> Element {
    let mut label = use_signal(String::new);
    let mut pending = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        Dialog {
            open: true,
            on_open_change: move |open: bool| if !open { on_close.call(()) },
            form {
                class: "grid gap-3",
                onsubmit: move |event| {
                    event.prevent_default();
                    let request = CreateTenantRequest { label: label() };
                    pending.set(true);
                    error.set(None);
                    spawn(async move {
                        match create_tenant(request).await {
                            Ok(()) => reload(),
                            Err(message) => error.set(Some(message)),
                        }
                        pending.set(false);
                    });
                },
                DialogTitle { "新建租户" }
                DialogDescription { "新租户会建立独立的插件组合。" }
                label { r#for: "tenant-label", "租户名称" }
                Input {
                    id: "tenant-label",
                    aria_label: "租户名称",
                    value: label(),
                    oninput: move |event: FormEvent| label.set(event.value()),
                }
                if let Some(message) = error() {
                    p { role: "alert", "{message}" }
                }
                footer { class: "flex justify-end gap-2",
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    Button { r#type: "submit", disabled: pending(), "创建" }
                }
            }
        }
    }
}

async fn load_tenants() -> Result<Vec<TenantItem>, String> {
    let response = gloo_net::http::Request::get("/api/tenants")
        .send()
        .await
        .map_err(|error| error.to_string())?;
    decode::<Vec<TenantItem>>(response).await
}

async fn create_tenant(request: CreateTenantRequest) -> Result<(), String> {
    let response = gloo_net::http::Request::post("/api/tenants")
        .json(&request)
        .map_err(|error| error.to_string())?
        .send()
        .await
        .map_err(|error| error.to_string())?;
    decode::<TenantItem>(response).await.map(|_| ())
}

async fn switch_tenant(tenant_id: String) -> Result<(), String> {
    let response = gloo_net::http::Request::post("/api/tenants/switch")
        .json(&SwitchTenantRequest { tenant_id })
        .map_err(|error| error.to_string())?
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if response.ok() {
        Ok(())
    } else {
        decode_error(response).await
    }
}

async fn decode<T: serde::de::DeserializeOwned>(
    response: gloo_net::http::Response,
) -> Result<T, String> {
    if !response.ok() {
        return decode_error(response).await;
    }
    response
        .json::<TenantResponse<T>>()
        .await
        .map(|response| response.data)
        .map_err(|error| error.to_string())
}

async fn decode_error<T>(response: gloo_net::http::Response) -> Result<T, String> {
    let body = response.text().await.unwrap_or_default();
    Err(serde_json::from_str::<TenantErrorResponse>(&body)
        .map(|response| response.error)
        .unwrap_or(body))
}

fn reload() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

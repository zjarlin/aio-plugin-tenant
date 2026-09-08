use az_dioxus_admin_shell::{
    ApplicationAccountItem, ApplicationAccountPlugin, ApplicationPage, ApplicationPlugin,
    ApplicationScene,
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
    rsx! {
        section {
            h2 { "租户管理" }
            p { "当前租户：默认租户" }
            p { "每个租户拥有独立的插件组合与锁定版本。" }
        }
    }
}

mod http;
mod page;

use az_dioxus_admin_shell::{
    ApplicationAccountItem, ApplicationAccountPlugin, ApplicationPage, ApplicationPlugin,
    ApplicationScene,
};
use dill::CatalogBuilder;

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
            render: page::TenantPage,
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

pub mod header;
pub mod sidebar;
pub mod toggle;

use std::hash::{DefaultHasher, Hash, Hasher};

use leptos::*;
use leptos_meta::Body;
use leptos_router::Outlet;

use crate::{
    components::{
        layout::{header::Header, sidebar::SideBar, toggle::ToggleNavigation},
        messages::modal::Modal,
    },
    core::schema::{Schema, SchemaType},
};

pub struct LayoutBuilder {
    pub menu_items: Vec<MenuItem>,
    base_path: String,
    chain: Vec<MenuItem>,
    last_path: String,
}

pub struct Route {
    pub path: String,
    pub view: Box<dyn Fn() -> View + 'static>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MenuItem {
    pub name: String,
    pub route: Option<String>,
    pub icon: Option<View>,
    pub children: Vec<MenuItem>,
}

#[component]
pub fn Layout(
    menu_items: Vec<MenuItem>,
    #[prop(into)] is_admin: MaybeSignal<bool>,
) -> impl IntoView {
    let menu_items_toggle = menu_items.clone();
    let show_sidebar = create_rw_signal(false);

    view! {
        <Body class="bg-gray-50 dark:bg-slate-900"/>
        <Modal/>
        <Header is_admin/>
        <ToggleNavigation menu_items show_sidebar/>
        <SideBar menu_items=menu_items_toggle show_sidebar/>
        <div class="w-full pt-10 px-4 sm:px-6 md:px-8 lg:ps-72">
            <Outlet/>
        </div>
    }
}

impl LayoutBuilder {
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            menu_items: vec![],
            base_path: base_path.into(),
            chain: vec![],
            last_path: "".to_string(),
        }
    }

    pub fn create_from_schema(mut self, schema: &Schema) -> Self {
        let route = if matches!(
            schema.typ,
            SchemaType::Record { .. } | SchemaType::Entry { .. }
        ) {
            format!("/settings/{}", schema.id)
        } else {
            format!("/settings/{}/edit", schema.id)
        };
        self.menu_items.push(MenuItem {
            name: schema.list.title.into(),
            route: route.into(),
            ..Default::default()
        });
        self
    }

    pub fn create(mut self, name: impl Into<String>) -> Self {
        self.chain.push(MenuItem {
            name: name.into(),
            ..Default::default()
        });
        self
    }

    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.chain.last_mut().unwrap().icon = Some(icon.into_view());
        self
    }

    pub fn route(mut self, route: impl Into<String>) -> Self {
        self.last_path = route.into();
        self.chain.last_mut().unwrap().route =
            Some(format!("{}{}", self.base_path, self.last_path));
        self
    }

    pub fn raw_route(mut self, route: impl Into<String>) -> Self {
        self.chain.last_mut().unwrap().route = route.into().into();
        self
    }

    pub fn insert(mut self) -> Self {
        let menu_item = self.chain.pop().unwrap();
        if let Some(parent_menu_item) = self.chain.last_mut() {
            parent_menu_item.children.push(menu_item);
        } else {
            self.menu_items.push(menu_item);
        }
        self
    }
}

impl MenuItem {
    pub fn id(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        self.route.hash(&mut hasher);
        self.children.len().hash(&mut hasher);
        hasher.finish().to_string()
    }
}

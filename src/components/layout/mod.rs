pub mod header;
pub mod sidebar;
pub mod toggle;

use std::hash::{DefaultHasher, Hash, Hasher};

use leptos::*;
use leptos_meta::Body;
use leptos_router::Outlet;

use crate::components::{
    layout::{header::Header, sidebar::SideBar, toggle::ToggleNavigation},
    messages::modal::Modal,
};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MenuItem {
    pub name: String,
    pub route: Option<String>,
    pub match_route: Option<String>,
    pub icon: Option<String>,
    pub children: Vec<MenuItem>,
}

#[component]
pub fn Layout(menu_items: Vec<MenuItem>) -> impl IntoView {
    let menu_items_toggle = menu_items.clone();
    let show_sidebar = create_rw_signal(false);

    view! {
        <Body class="bg-gray-50 dark:bg-slate-900"/>
        <Modal/>
        <Header/>
        <ToggleNavigation menu_items show_sidebar/>
        <SideBar menu_items=menu_items_toggle show_sidebar/>
        <div class="w-full pt-10 px-4 sm:px-6 md:px-8 lg:ps-72">
            <Outlet/>
        </div>
    }
}

impl MenuItem {
    pub fn parent_with_icon(
        name: impl Into<String>,
        icon: impl Into<String>,
        children: Vec<MenuItem>,
    ) -> Self {
        Self {
            name: name.into(),
            route: None,
            icon: Some(icon.into()),
            children,
            match_route: None,
        }
    }

    pub fn parent(name: impl Into<String>, children: Vec<MenuItem>) -> Self {
        Self {
            name: name.into(),
            route: None,
            icon: None,
            children,
            match_route: None,
        }
    }

    pub fn child(name: impl Into<String>, route: impl Into<String>) -> Self {
        let route = route.into();
        Self {
            name: name.into(),
            route: Some(route.clone()),
            icon: None,
            children: vec![],
            match_route: Some(route),
        }
    }

    pub fn child_with_icon(
        name: impl Into<String>,
        icon: impl Into<String>,
        route: impl Into<String>,
    ) -> Self {
        let route = route.into();
        Self {
            name: name.into(),
            route: Some(route.clone()),
            icon: Some(icon.into()),
            children: vec![],
            match_route: Some(route),
        }
    }

    pub fn id(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        self.route.hash(&mut hasher);
        self.icon.hash(&mut hasher);
        self.children.len().hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn with_match_route(mut self, match_route: impl Into<String>) -> Self {
        self.match_route = Some(match_route.into());
        self
    }
}

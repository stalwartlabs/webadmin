use leptos::*;
use leptos_meta::Style;
use leptos_router::{use_location, use_navigate, Outlet};
use thaw::*;

use crate::components::main::{alert::Alerts, header::SiteHeader};

#[component]
pub fn ManagePage() -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();

    let select_name = create_rw_signal(String::new());
    create_effect(move |_| {
        let mut pathname = location.pathname.get();
        let name = if pathname.starts_with("/manage/") {
            pathname.drain(12..).collect()
        } else {
            String::new()
        };
        select_name.set(name);
    });

    _ = select_name.watch(move |name| {
        let pathname = location.pathname.get_untracked();
        if !pathname.eq(&format!("/manage/{name}")) {
            navigate(&format!("/manage/{name}"), Default::default());
        }
    });
    view! {
        <Style>
            "
            .demo-components__component {
                width: 896px;
                margin: 0 auto;
            }
            .demo-md-table-box {
                overflow: auto;
            }
            .collapse-padding {
                padding: 16px;
            }
            .font-bold {
                font-weight: bold;
            }
            .font-normal {
                font-weight: normal;
            }
            @media screen and (max-width: 1200px) {
                .demo-components__sider {
                    display: none;
                }
                .demo-components__component {
                    width: 100%;
                }
            }
            "
        </Style>
        <crate::components::main::modal::Modal></crate::components::main::modal::Modal>

        <Layout position=LayoutPosition::Absolute>
            <SiteHeader/>
            <Layout has_sider=true position=LayoutPosition::Absolute style="top: 64px;">
                <LayoutSider class="demo-components__sider">
                    <Collapse class="collapse-padding" accordion=true>
                        {manage_menu_items().into_view()}
                    </Collapse>
                </LayoutSider>
                <Layout style="padding: 8px 12px 28px; overflow-y: auto;">
                    <Outlet/>
                </Layout>
            </Layout>
        </Layout>
    }
}

pub(crate) struct MenuGroupOption {
    pub label: String,
    pub children: Vec<MenuItemOption>,
}

impl IntoView for MenuGroupOption {
    fn into_view(self) -> View {
        let Self { label, children } = self;
        let key = label.to_lowercase().replace(' ', "");
        view! {
            <CollapseItem class="font-bold" title=label key=key>
                <Menu>{children.into_iter().map(|v| v.into_view()).collect_view()}</Menu>
            </CollapseItem>
        }
    }
}

pub(crate) struct MenuItemOption {
    pub label: String,
    pub value: String,
}

impl IntoView for MenuItemOption {
    fn into_view(self) -> View {
        let Self { label, value } = self;
        view! { <MenuItem class="font-normal" key=value label/> }
    }
}

/*

 - Directory
   - Accounts
   - Groups
   - Mailing Lists
   - Domains
 - Queues
   - Messages
   - Reports

*/

pub(crate) fn manage_menu_items() -> Vec<MenuGroupOption> {
    vec![
        MenuGroupOption {
            label: "Directory".into(),
            children: vec![
                MenuItemOption {
                    value: "accounts".into(),
                    label: "Accounts".into(),
                },
                MenuItemOption {
                    value: "groups".into(),
                    label: "Groups".into(),
                },
                MenuItemOption {
                    value: "lists".into(),
                    label: "Lists".into(),
                },
                MenuItemOption {
                    value: "domains".into(),
                    label: "Domains".into(),
                },
            ],
        },
        MenuGroupOption {
            label: "Queues".into(),
            children: vec![
                MenuItemOption {
                    value: "messages".into(),
                    label: "Messages".into(),
                },
                MenuItemOption {
                    value: "reports".into(),
                    label: "Reports".into(),
                },
            ],
        },
    ]
}

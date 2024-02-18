use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use thaw::*;

use crate::{
    components::main::{manage::ManagePage, notfound::NotFound},
    pages::directory::accounts::list::AccountList,
};

pub mod components;
pub mod core;
pub mod pages;

fn main() {
    console_error_panic_hook::set_once();

    leptos::mount_to_body(|| view! { <App/> })
}

#[component]
pub fn App() -> impl IntoView {
    let is_routing = create_rw_signal(false);
    let set_is_routing = SignalSetter::map(move |is_routing_data| {
        is_routing.set(is_routing_data);
    });
    provide_meta_context();

    view! {
        <Router set_is_routing>
            <TheProvider>
                <TheRouter is_routing/>
            </TheProvider>
        </Router>
    }
}

#[component]
fn TheRouter(is_routing: RwSignal<bool>) -> impl IntoView {
    let loading_bar = use_loading_bar();
    _ = is_routing.watch(move |is_routing| {
        if *is_routing {
            loading_bar.start();
        } else {
            loading_bar.finish();
        }
    });

    view! {
        <Routes>
            <Route path="/" view=NotFound/>
            <Route path="/404" view=move || view! { <NotFound/> }/>
            <Route path="/manage" view=ManagePage>
                <Route path="/accounts" view=AccountList/>

            </Route>
        </Routes>
    }
}

#[component]
fn TheProvider(children: Children) -> impl IntoView {
    fn use_query_value(key: &str) -> Option<String> {
        let query_map = use_query_map();
        query_map.with_untracked(|query| query.get(key).cloned())
    }
    let theme = use_query_value("theme").map_or_else(Theme::light, |name| {
        if name == "light" {
            Theme::light()
        } else if name == "dark" {
            Theme::dark()
        } else {
            Theme::light()
        }
    });
    let theme = create_rw_signal(theme);

    view! {
        <ThemeProvider theme>
            <GlobalStyle/>
            <MessageProvider>
                <LoadingBarProvider>{children()}</LoadingBarProvider>
            </MessageProvider>
        </ThemeProvider>
    }
}

#[component]
fn Home() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div class="my-0 mx-auto max-w-3xl text-center">
            <h2 class="p-6 text-4xl">"Welcome to Leptos with Tailwind"</h2>
            <p class="px-10 pb-10 text-left">
                "Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."
            </p>
            <button
                class="bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
                on:click=move |_| set_count.update(|count| *count += 1)
            >
                "Something's here | "
                {move || if count() == 0 { "Click me!".to_string() } else { count().to_string() }}

                " | Some more text"
            </button>
        </div>
    }
}

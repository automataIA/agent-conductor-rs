use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod api;
mod components;

use components::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/conductor-ui.css"/>
        <Title text="Agent Conductor"/>

        <Router>
            <nav class="navbar">
                <div class="container">
                    <h1>"🎭 Agent Conductor"</h1>
                    <div class="nav-links">
                        <A href="/">"Templates"</A>
                        <A href="/sessions">"Sessions"</A>
                    </div>
                </div>
            </nav>

            <main class="container">
                <Routes>
                    <Route path="/" view=TemplatesPage/>
                    <Route path="/sessions" view=SessionsPage/>
                    <Route path="/sessions/:id" view=SessionDetailPage/>
                </Routes>
            </main>

            <footer class="footer">
                <div class="container">
                    <p>"Built with Rust + Leptos + WASM"</p>
                </div>
            </footer>
        </Router>
    }
}

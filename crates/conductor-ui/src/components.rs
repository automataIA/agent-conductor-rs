//! UI Components

use crate::api;
use leptos::*;
use leptos_router::*;
use std::collections::HashMap;

/// Templates page - list all available workflow templates
#[component]
pub fn TemplatesPage() -> impl IntoView {
    let templates = create_resource(|| (), |_| async { api::list_templates().await });
    let navigate = use_navigate();

    let create_session = create_action(move |template_name: &String| {
        let template_name = template_name.clone();
        let navigate = navigate.clone();
        async move {
            match api::create_session(template_name).await {
                Ok(response) => {
                    navigate(&format!("/sessions/{}", response.session_id), Default::default());
                }
                Err(e) => {
                    logging::error!("Failed to create session: {}", e);
                }
            }
        }
    });

    view! {
        <div class="page">
            <h2>"Workflow Templates"</h2>
            <p class="subtitle">"Select a template to create a new session"</p>

            <Suspense fallback=move || view! { <p>"Loading templates..."</p> }>
                {move || templates.get().map(|result| {
                    match result {
                        Ok(templates) => {
                            view! {
                                <div class="template-list">
                                    {templates.into_iter().map(|template| {
                                        let name = template.name.clone();
                                        view! {
                                            <div class="card template-card">
                                                <h3>{&template.name}</h3>
                                                <p>{&template.description}</p>
                                                <button
                                                    class="btn btn-primary"
                                                    on:click=move |_| create_session.dispatch(name.clone())
                                                >
                                                    "Create Session"
                                                </button>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        }
                        Err(e) => {
                            view! {
                                <div class="error">
                                    <p>"Error loading templates: " {e}</p>
                                </div>
                            }.into_view()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

/// Sessions page - list all active sessions
#[component]
pub fn SessionsPage() -> impl IntoView {
    let sessions = create_resource(|| (), |_| async { api::list_sessions().await });

    let refresh = move || sessions.refetch();

    view! {
        <div class="page">
            <div class="page-header">
                <div>
                    <h2>"Active Sessions"</h2>
                    <p class="subtitle">"Manage your workflow sessions"</p>
                </div>
                <button class="btn btn-secondary" on:click=move |_| refresh()>
                    "🔄 Refresh"
                </button>
            </div>

            <Suspense fallback=move || view! { <p>"Loading sessions..."</p> }>
                {move || sessions.get().map(|result| {
                    match result {
                        Ok(sessions) => {
                            if sessions.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <p>"No active sessions"</p>
                                        <A href="/" class="btn btn-primary">"Create New Session"</A>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="session-list">
                                        {sessions.into_iter().map(|session| {
                                            let session_id = session.session_id.clone();
                                            view! {
                                                <A href=format!("/sessions/{}", &session_id) class="card session-card">
                                                    <div class="session-header">
                                                        <h3>{&session.session_id}</h3>
                                                        <span class="badge">{&session.template_name}</span>
                                                    </div>
                                                    <p class="session-stats">
                                                        "Executions: " {session.execution_count}
                                                    </p>
                                                </A>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_view()
                            }
                        }
                        Err(e) => {
                            view! {
                                <div class="error">
                                    <p>"Error loading sessions: " {e}</p>
                                </div>
                            }.into_view()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

/// Session detail page - execute workflows and view results
#[component]
pub fn SessionDetailPage() -> impl IntoView {
    let params = use_params_map();
    let session_id = move || params.get().get("id").cloned().unwrap_or_default();
    let navigate = use_navigate();

    let session = create_resource(
        move || session_id(),
        |id| async move { api::get_session(&id).await },
    );

    let (input_value, set_input_value) = create_signal(String::new());
    let (result, set_result) = create_signal(Option::<HashMap<String, serde_json::Value>>::None);
    let (executing, set_executing) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    let execute = move |_| {
        let id = session_id();
        let input = input_value.get();

        spawn_local(async move {
            set_executing.set(true);
            set_error.set(None);

            // Parse input as JSON or use default state
            let state = if input.trim().is_empty() {
                HashMap::new()
            } else {
                match serde_json::from_str::<HashMap<String, serde_json::Value>>(&input) {
                    Ok(s) => s,
                    Err(e) => {
                        set_error.set(Some(format!("Invalid JSON: {}", e)));
                        set_executing.set(false);
                        return;
                    }
                }
            };

            match api::execute_workflow(&id, state).await {
                Ok(response) => {
                    set_result.set(Some(response.result));
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }

            set_executing.set(false);
        });
    };

    let delete_session = move |_| {
        let id = session_id();
        let navigate = navigate.clone();
        spawn_local(async move {
            if let Err(e) = api::delete_session(&id).await {
                logging::error!("Failed to delete session: {}", e);
            } else {
                navigate("/sessions", Default::default());
            }
        });
    };

    view! {
        <div class="page">
            <Suspense fallback=move || view! { <p>"Loading session..."</p> }>
                {move || session.get().map(|result| {
                    match result {
                        Ok(session) => {
                            view! {
                                <div>
                                    <div class="page-header">
                                        <div>
                                            <h2>"Session: " {&session.session_id}</h2>
                                            <div class="session-meta">
                                                <span class="badge">{&session.template_name}</span>
                                                <span class="meta-item">
                                                    "Executions: " {session.execution_count}
                                                </span>
                                            </div>
                                        </div>
                                        <button
                                            class="btn btn-danger"
                                            on:click=delete_session
                                        >
                                            "🗑️ Delete"
                                        </button>
                                    </div>

                                    <div class="execution-panel">
                                        <div class="card">
                                            <h3>"Execute Workflow"</h3>
                                            <p class="help-text">
                                                "Enter initial state as JSON (optional, leave empty for default):"
                                            </p>

                                            <textarea
                                                class="input-json"
                                                placeholder=r#"{"key": "value"}"#
                                                prop:value=move || input_value.get()
                                                on:input=move |ev| set_input_value.set(event_target_value(&ev))
                                            />

                                            <button
                                                class="btn btn-primary"
                                                on:click=execute
                                                disabled=move || executing.get()
                                            >
                                                {move || if executing.get() { "⏳ Executing..." } else { "▶️ Execute" }}
                                            </button>

                                            {move || error.get().map(|e| view! {
                                                <div class="error-box">
                                                    <strong>"Error:"</strong>
                                                    <p>{e}</p>
                                                </div>
                                            })}
                                        </div>

                                        {move || result.get().map(|res| view! {
                                            <div class="card result-card">
                                                <h3>"Result"</h3>
                                                <pre class="result-json">
                                                    {serde_json::to_string_pretty(&res).unwrap_or_default()}
                                                </pre>
                                            </div>
                                        })}
                                    </div>
                                </div>
                            }.into_view()
                        }
                        Err(e) => {
                            view! {
                                <div class="error">
                                    <p>"Error loading session: " {e}</p>
                                    <A href="/sessions" class="btn btn-secondary">"← Back to Sessions"</A>
                                </div>
                            }.into_view()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

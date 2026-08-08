#[cfg(target_arch = "wasm32")]
mod articles;
#[cfg(target_arch = "wasm32")]
mod boot;
#[cfg(target_arch = "wasm32")]
mod panel;
/// The static render emits the same footer button, but there is no document to
/// open a panel in until the browser build attaches one.
#[cfg(not(target_arch = "wasm32"))]
mod panel {
    pub fn open() {}
}
mod projects;
mod runtime;

use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlInputElement, KeyboardEvent};

use projects::{CATEGORIES, PROJECTS};
use runtime::PRODUCTION_WASM_SIZE;

#[cfg(target_arch = "wasm32")]
pub use boot::run;

#[component]
pub fn App() -> impl IntoView {
    let category = RwSignal::new("All".to_string());
    let query = RwSignal::new(String::new());
    let active = RwSignal::new(0usize);
    let expanded = RwSignal::new(None::<usize>);

    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if expanded.get().is_some() {
                runtime::focus_element("case-close");
            }
        });

        let key_handler = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
            let typing = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.active_element())
                .map(|element| element.tag_name() == "INPUT")
                .unwrap_or(false);

            if event.key() == "Escape" {
                expanded.set(None);
                panel::close();
                return;
            }
            if typing {
                return;
            }
            match event.key().as_str() {
                "/" => {
                    event.prevent_default();
                    if let Some(input) = web_sys::window()
                        .and_then(|window| window.document())
                        .and_then(|document| document.get_element_by_id("project-search"))
                        .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
                    {
                        let _ = input.focus();
                    }
                }
                "j" | "J" => active.update(|index| *index = (*index + 1) % PROJECTS.len()),
                "k" | "K" => {
                    active.update(|index| *index = (*index + PROJECTS.len() - 1) % PROJECTS.len())
                }
                "Enter" => expanded.set(Some(active.get_untracked())),
                "s" | "S" => panel::toggle(),
                _ => {}
            }
        });
        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback("keydown", key_handler.as_ref().unchecked_ref());
            key_handler.forget();
        }
    }

    view! {
        <div class="site-shell">
            <a class="skip-link" href="#main-content">"Skip to main content"</a>

            <main id="main-content" tabindex="-1">
                <section id="top" class="hero section-grid">
                    <div class="section-index">"00"</div>
                    <div class="hero-content">
                        <p class="eyebrow">"FABIO ELLENA · SENIOR STAFF SOFTWARE ENGINEER · TURIN"</p>
                        <h1>"SOFTWARE SHOULD "<br/><em>"SURVIVE"</em><br/>"CONTACT WITH PRODUCTION."</h1>
                        <div class="runtime-facts" aria-label="Application runtime facts">
                            <span>"LEPTOS SSG"</span>
                            <span>"RUST/WASM"</span>
                            <span>"STATIC HTML"</span>
                            // Deterministic at build time, overwritten in place
                            // once the browser can measure it — the same
                            // mechanism the diagnostics panel uses, so there is
                            // one path for runtime facts rather than two.
                            <span id="hero-wasm">{format!("{PRODUCTION_WASM_SIZE} WASM")}</span>
                            <span id="hero-boot">"BOOT STATIC"</span>
                        </div>
                        <div class="hero-bottom">
                            <p>"I build distributed systems, telemetry platforms, and local-first AI tools for problems where scale, evidence, and reliability are not optional."</p>
                            <div class="hero-actions">
                                <a class="button button-solid" href="#work">"Inspect systems ↓"</a>
                                <a class="button" href="mailto:ellena.fabio@gmail.com">"Start a conversation ↗"</a>
                            </div>
                        </div>
                    </div>
                </section>

                <section id="impact" class="impact-grid" aria-label="Selected impact">
                    <div class="impact-cell"><strong>"10M+"</strong><span>"connected vehicles supported"</span></div>
                    <div class="impact-cell"><strong>"100+"</strong><span>"B2B and B2C APIs shaped"</span></div>
                    <div class="impact-cell"><strong>"0→20+"</strong><span>"engineering organization growth"</span></div>
                    <div class="impact-cell"><strong>"100K+"</strong><span>"IoT devices operated"</span></div>
                </section>

                <section id="work" class="work section-grid">
                    <div class="section-index">"01"</div>
                    <div>
                        <div class="section-heading">
                            <div><p class="eyebrow">"SELECTED SYSTEMS"</p><h2>"THE WORK"</h2></div>
                            <p>"Four public projects. Each one built around real data, explicit constraints, and an inspectable technical argument."</p>
                        </div>

                        <div class="project-controls">
                            <div class="category-list" aria-label="Filter projects">
                                {CATEGORIES.into_iter().map(|item| view! {
                                    <button
                                        class:active=move || category.get() == item
                                        on:click=move |_| category.set(item.to_string())
                                    >{item}</button>
                                }).collect_view()}
                            </div>
                            <label class="search-box" for="project-search">
                                <span>"/"</span>
                                <input
                                    id="project-search"
                                    type="search"
                                    placeholder="FILTER STACK OR CONCEPT"
                                    prop:value=move || query.get()
                                    on:input=move |event| query.set(event_target_value(&event))
                                />
                            </label>
                        </div>

                        <div class="project-list">
                            {PROJECTS.into_iter().enumerate().map(|(index, project)| view! {
                                <article
                                    class="project-row"
                                    class:active=move || active.get() == index
                                    class:hidden=move || !projects::matches(project, &category.get(), &query.get())
                                    id=project.slug
                                >
                                    <button
                                        class="project-open"
                                        aria-controls="case-study"
                                        aria-expanded=move || (expanded.get() == Some(index)).to_string()
                                        aria-haspopup="dialog"
                                        on:click=move |_| { active.set(index); expanded.set(Some(index)); }
                                        aria-label=format!("Open {} case study", project.name)
                                    >
                                        <span class="project-number">{format!("{:02}", index + 1)}</span>
                                        <span class="project-name">{project.name}</span>
                                        <span class="project-stack">{project.stack}</span>
                                        <span class="project-arrow">"VIEW"</span>
                                    </button>
                                    <div class="project-summary">
                                        <p>{project.statement}</p>
                                        <div class="tag-list">{project.tags.iter().map(|tag| view! { <span>{*tag}</span> }).collect_view()}</div>
                                    </div>
                                </article>
                            }).collect_view()}
                        </div>
                        <p class="keyboard-hint">"KEYBOARD: J/K SELECT · ENTER INSPECT · / SEARCH · S SYSTEM · ESC CLOSE"</p>
                    </div>
                </section>

                <section class="principles section-grid">
                    <div class="section-index">"02"</div>
                    <div>
                        <div class="section-heading"><div><p class="eyebrow">"OPERATING PRINCIPLES"</p><h2>"HOW I BUILD"</h2></div></div>
                        <div class="principle-grid">
                            <article><span>"A"</span><h3>"MEASURE THE REAL SYSTEM"</h3><p>"Production distributions, traces, failure modes, and representative data beat architectural theatre."</p></article>
                            <article><span>"B"</span><h3>"MAKE EVIDENCE QUERYABLE"</h3><p>"Graphs, typed contracts, telemetry, and deterministic tests turn understanding into infrastructure."</p></article>
                            <article><span>"C"</span><h3>"SHIP THE SMALLEST SOLID THING"</h3><p>"A narrow production-grade loop creates more leverage than a broad demo that cannot be trusted."</p></article>
                        </div>
                    </div>
                </section>

                <section id="capabilities" class="capabilities section-grid">
                    <div class="section-index">"03"</div>
                    <div>
                        <div class="section-heading"><div><p class="eyebrow">"WHAT I WORK WITH"</p><h2>"THE STACK"</h2></div><p>"Eight years across languages, cloud platforms, and the messaging and observability tooling that keeps distributed systems fast, traceable, and honest."</p></div>
                        <div class="capability-grid">
                            <article><h3>"LANGUAGES"</h3><div class="cap-tags"><span>"Java"</span><span>"TypeScript"</span><span>"Rust"</span><span>"Python"</span><span>"SQL"</span></div></article>
                            <article><h3>"BACKEND"</h3><div class="cap-tags"><span>"Spring Boot"</span><span>"Quarkus"</span><span>"REST APIs"</span><span>"Event-Driven Architecture"</span><span>"API Design"</span></div></article>
                            <article><h3>"CLOUD & INFRASTRUCTURE"</h3><div class="cap-tags"><span>"AWS"</span><span>"Kubernetes"</span><span>"Docker"</span><span>"Serverless"</span><span>"Linux"</span></div></article>
                            <article><h3>"DATA & MESSAGING"</h3><div class="cap-tags"><span>"Kafka"</span><span>"Kinesis"</span><span>"SQS"</span><span>"MongoDB"</span><span>"DynamoDB"</span><span>"S3"</span></div></article>
                            <article><h3>"OBSERVABILITY"</h3><div class="cap-tags"><span>"OpenTelemetry"</span><span>"ELK Stack"</span><span>"CloudWatch"</span><span>"Distributed Tracing"</span></div></article>
                            <article><h3>"CI/CD & QUALITY"</h3><div class="cap-tags"><span>"GitHub Actions"</span><span>"TeamCity"</span><span>"Jenkins"</span><span>"SonarQube"</span><span>"Checkmarx"</span></div></article>
                            <article><h3>"AI & DEVELOPER PRODUCTIVITY"</h3><div class="cap-tags"><span>"Agentic Workflows"</span><span>"AI-Assisted Development"</span><span>"MCP Integrations"</span><span>"Developer Experience"</span></div></article>
                        </div>
                    </div>
                </section>

                <section id="experience" class="experience section-grid">
                    <div class="section-index">"04"</div>
                    <div>
                        <div class="section-heading"><div><p class="eyebrow">"2017 → NOW"</p><h2>"EXPERIENCE"</h2></div><p>"From on-device ML to platforms serving millions of connected vehicles."</p></div>
                        <div class="timeline">
                            <article><time>"2024—NOW"</time><div><h3>"Senior Staff Software Engineer"</h3><p>"Stellantis · Connected Services"</p></div><p>"Merging two decades of divergent FCA and PSA APIs into a single connected-services platform. I set the technical direction for 100+ B2B and consumer services, made the fleet observable with OpenTelemetry and ELK so millions of vehicles can be traced and debugged in production, and moved delivery off TeamCity onto GitHub Actions — releases stopped being events and became routine."</p></article>
                            <article><time>"2023—2024"</time><div><h3>"Staff Software Engineer"</h3><p>"Stellantis · Connected Services"</p></div><p>"Joined to turn a green-field team into an organization. Grew it from zero to 20+ engineers, wrote the developer-experience and delivery standards it still runs on, and helped ship the Jeep Avenger connected-services launch across every domain it touched."</p></article>
                            <article><time>"2022—2023"</time><div><h3>"Technical Lead"</h3><p>"FCA · Connected Services"</p></div><p>"Owned the telemetry platform ingesting live data from 10M+ vehicles, and wired the car into the world around it — EV charging networks, mapping providers, ecosystem partners — while trimming the AWS bill through architecture the scale finally justified."</p></article>
                            <article><time>"2018—2022"</time><div><h3>"Software Engineer"</h3><p>"Concept Reply"</p></div><p>"Built the cloud-native backbone for Grohe's smart-home ecosystem: 100K+ connected devices, their telemetry, device management, and enterprise integrations. On the side, I delivered a Beretta Android app that turned raw performance data into guided, real-time training."</p></article>
                            <article><time>"2017—2018"</time><div><h3>"ML Engineer Intern"</h3><p>"Docapost · Nice"</p></div><p>"Started where the data was messiest: an on-device model classifying documents in real time under tight constraints, paired with a semi-automated labeling pipeline that cut manual annotation roughly in half."</p></article>
                        </div>
                    </div>
                </section>

                <section class="education section-grid">
                    <div class="section-index">"05"</div>
                    <div class="education-layout">
                        <div>
                            <p class="eyebrow">"FOUNDATIONS"</p>
                            <h2>"TWO MASTER'S. ONE SYSTEMS MINDSET."</h2>
                            <div class="publication">
                                <p class="cred-label">"PUBLICATION"</p>
                                <strong>"CityMUS: Music Recommendation When Exploring a City"</strong>
                                <p>"A context-aware recommender that fuses DBpedia knowledge graphs with the Spotify API to generate location-based, personalized listening as you move through a city."</p>
                                <div class="pub-links">
                                    <a href="https://ceur-ws.org/Vol-1963/paper569.pdf" target="_blank" rel="noreferrer">"Paper · PDF ↗"</a>
                                    <a href="https://docs.google.com/presentation/d/1UqjRT2UrgYTE65wLAtynNRMo0Bks29aV3gOQjVfmrdU/edit?usp=sharing" target="_blank" rel="noreferrer">"Slides ↗"</a>
                                    <a href="https://github.com/D2KLab/CityMUS" target="_blank" rel="noreferrer">"Code ↗"</a>
                                </div>
                            </div>
                        </div>
                        <div>
                            <p class="cred-label">"DEGREES"</p>
                            <div class="degree-list">
                                <article><strong>"M.Sc. Data Science"</strong><span>"EURECOM · TÉLÉCOM PARISTECH · GPA 4.0"</span></article>
                                <article><strong>"M.Sc. Computer Engineering"</strong><span>"POLITECNICO DI TORINO · 110 CUM LAUDE"</span></article>
                                <article><strong>"B.Sc. Computer Engineering"</strong><span>"POLITECNICO DI TORINO"</span></article>
                            </div>
                            <p class="cred-label">"CERTIFICATIONS"</p>
                            <div class="degree-list">
                                <article><strong>"Nanodegree · Machine Learning DevOps"</strong><span>"UDACITY · 2025"</span></article>
                                <article><strong>"Nanodegree · Generative AI"</strong><span>"UDACITY · 2024"</span></article>
                            </div>
                        </div>
                    </div>
                </section>

                <section id="interests" class="interests section-grid">
                    <div class="section-index">"06"</div>
                    <div>
                        <div class="section-heading"><div><p class="eyebrow">"OFF THE CLOCK"</p><h2>"WHERE THE WORK COMES FROM"</h2></div><p>"The projects aren't separate from the hobbies. The mountains and the night sky are where the problems start; the repositories and the writing are where they end up."</p></div>
                        <div class="interest-grid">
                            <article>
                                <p class="eyebrow">"ALTITUDE"</p>
                                <h3>"ALPINE TREKKING"</h3>
                                <p>"The Alps are where the terrain work started. Every GPX track I record is elevation, slope, and forest cover waiting to be sampled — which is exactly what Ridgeline turns into a 3D artifact. The mountains are the field test; the code is the souvenir."</p>
                                <div class="interest-links">
                                    <a href="#ridgeline">"Ridgeline ↓"</a>
                                    <a href="https://github.com/fblln/ridgeline" target="_blank" rel="noreferrer">"Repository ↗"</a>
                                </div>
                            </article>
                            <article>
                                <p class="eyebrow">"SIGNAL"</p>
                                <h3>"ASTROPHOTOGRAPHY"</h3>
                                <p>"Astrophotography is telemetry with a longer exposure: stacking faint signal out of noise, trusting your calibration frames, and staying honest about what the data can and can't show. The same instincts behind a clean night-sky image build a trustworthy race-telemetry pipeline."</p>
                                <div class="interest-links">
                                    <a href="#race-telemetry">"Race Telemetry ↓"</a>
                                    <a href="/articles/">"Field notes ↗"</a>
                                </div>
                            </article>
                        </div>
                    </div>
                </section>

                <section id="contact" class="contact">
                    <img src="/assets/fabio.webp" alt="Fabio Ellena" width="460" height="460" />
                    <div>
                        <p class="eyebrow">"OPEN TO HARD PROBLEMS"</p>
                        <h2>"LET'S BUILD SOMETHING THAT HAS TO WORK."</h2>
                        <div class="contact-links">
                            <a href="mailto:ellena.fabio@gmail.com">"EMAIL ↗"</a>
                            <a href="https://www.linkedin.com/in/fabioellena/" target="_blank" rel="noreferrer">"LINKEDIN ↗"</a>
                            <a href="https://github.com/fblln" target="_blank" rel="noreferrer">"GITHUB ↗"</a>
                        </div>
                    </div>
                </section>
            </main>

            <footer>
                <span>"© 2026 FABIO ELLENA"</span>
                <a href="/articles/">"WRITING ↗"</a>
                <span>"FULL RUST · LEPTOS · WASM32"</span>
                // The panel lives outside the hydration root, so this reaches it
                // through the same DOM entry point the topbar button uses.
                <button on:click=move |_| panel::open()>"INSPECT RUNTIME [S]"</button>
            </footer>

            <div id="case-study" class="case-overlay" class:open=move || expanded.get().is_some() hidden=move || expanded.get().is_none()>
                <button class="overlay-backdrop" aria-label="Close case study" on:click=move |_| expanded.set(None)></button>
                <article class="case-panel" role="dialog" aria-modal="true" aria-labelledby="case-title">
                    <div class="panel-head"><span>{move || expanded.get().map(|index| format!("CASE/{:02}", index + 1)).unwrap_or_default()}</span><button id="case-close" on:click=move |_| expanded.set(None)>"CLOSE [ESC]"</button></div>
                    <div class="case-body">
                        <p class="eyebrow">{move || expanded.get().map(|index| PROJECTS[index].stack).unwrap_or("")}</p>
                        <h2 id="case-title">{move || expanded.get().map(|index| PROJECTS[index].name).unwrap_or("")}</h2>
                        <p class="case-statement">{move || expanded.get().map(|index| PROJECTS[index].statement).unwrap_or("")}</p>
                        <div class="case-metric"><strong>{move || expanded.get().map(|index| PROJECTS[index].metric).unwrap_or("")}</strong><span>{move || expanded.get().map(|index| PROJECTS[index].metric_label).unwrap_or("")}</span></div>
                        <p>{move || expanded.get().map(|index| PROJECTS[index].detail).unwrap_or("")}</p>
                        <blockquote class:hidden=move || expanded.get().map(|index| PROJECTS[index].evidence.is_empty()).unwrap_or(true)>{move || expanded.get().map(|index| PROJECTS[index].evidence).unwrap_or("")}</blockquote>
                        <div class="case-visual" class:graph=move || expanded.get().map(|index| PROJECTS[index].slug == "lithograph").unwrap_or(false)>
                            <img
                                class:hidden=move || expanded.get().map(|index| PROJECTS[index].image.is_empty()).unwrap_or(true)
                                src=move || expanded.get().map(|index| PROJECTS[index].image).unwrap_or("")
                                alt=move || expanded.get().map(|index| PROJECTS[index].image_alt).unwrap_or("")
                            />
                            <svg class="graph-visual" viewBox="0 0 720 300" role="img" aria-label="Semantic graph with connected code entities">
                                <g class="edges"><path d="M95 155L245 65L390 145L540 65L640 185"/><path d="M95 155L255 240L390 145L525 245L640 185"/><path d="M245 65L255 240M540 65L525 245"/></g>
                                <g class="nodes"><circle cx="95" cy="155" r="24"/><circle cx="245" cy="65" r="18"/><circle cx="255" cy="240" r="18"/><circle cx="390" cy="145" r="30"/><circle cx="540" cy="65" r="18"/><circle cx="525" cy="245" r="18"/><circle cx="640" cy="185" r="24"/></g>
                            </svg>
                        </div>
                        <a class="button button-solid" href=move || expanded.get().map(|index| PROJECTS[index].repo).unwrap_or("") target="_blank" rel="noreferrer">"INSPECT REPOSITORY ↗"</a>
                    </div>
                </article>
            </div>
        </div>
    }
}

/// Produces the exact HTML shape that the hydrate build expects. The build-time
/// site generator calls this native entry point, so portfolio content ships in
/// `index.html` and remains readable before (or without) WebAssembly.
#[cfg(feature = "ssr")]
pub fn render_static_app() -> String {
    view! { <App /> }.to_html()
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ssr")]
    use super::render_static_app;

    /// The hydration root must carry the portfolio's own content and Leptos'
    /// markers, and must *not* carry the shared chrome — that ships outside
    /// `#app` so both surfaces can render it from one source.
    #[cfg(feature = "ssr")]
    #[test]
    fn static_render_contains_content_and_hydration_markers() {
        let html = render_static_app();

        assert!(html.contains("SOFTWARE SHOULD"));
        assert!(html.contains("<!>"));
        assert!(!html.contains("class=\"topbar\""));
        assert!(!html.contains("id=\"system-panel\""));
    }
}

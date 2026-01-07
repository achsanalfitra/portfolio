use dioxus::prelude::*;
use gloo_timers::future::sleep;
use rsmg_core::prim::stack::{LinkedStack, LinkedStackNode};
use std::sync::LazyLock;
use std::time::Duration;

// The singleton anchor using LazyLock for runtime initialization
static MAGIC_DATA: LazyLock<LinkedStack<i32>> = LazyLock::new(LinkedStack::new);

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut stack_items = use_signal(|| vec![]);

    let handle_pop = move |_| {
        spawn(async move {
            // 1. Atomic POP from rsmg_core
            if let Ok(Some(_)) = MAGIC_DATA.pop() {
                let new_len = stack_items.read().len().saturating_sub(1);
                
                // 2. Snapshot Update
                if new_len == 0 {
                    stack_items.set(vec![]);
                } else {
                    // Creating a large Vec can be O(n). Spawning ensures
                    // this doesn't block a frame-paint if new_len is large.
                    stack_items.set(vec![new_len as i32; new_len]);
                }
            }
        });
    };

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Hero {}

        main { class: "container",
            div { id: "magic-area",
                div { class: "engine-spec",
                    div { class: "spec-line",
                        span { class: "spec-label", "ENGINE:" }
                        span { class: "spec-value", "rsmg_core::prim::stack::LinkedStack" }
                        a {
                            class: "small-link-chip",
                            href: "https://crates.io/crates/rsmg_core/0.1.0-alpha.1",
                            target: "_blank",
                            "0.1.0-alpha.1"
                        }
                    }
                    p { class: "spec-description",
                        "This is a magical counter that grows in size as the counter increments. "
                        "The operation takes advantage of workers doing atomic ops in the background "
                        "and the renderer loop takes a snapshot of it."
                    }
                    div { class: "bonus-challenge",
                        "BONUS CHALLENGE: Try to make your browser go \"Aw snap!\""
                    }
                }

            div { class: "controls",
                    button {
                        class: "btn btn-inc",
                        onclick: move |_| {
                            spawn(async move {
                                // 1. Perform the atomic work immediately
                                let current_val = stack_items.read().len() as i32 + 1;
                                MAGIC_DATA.push(LinkedStackNode::new(current_val));
                                
                                // 2. Sync the UI snapshot
                                stack_items.set(vec![current_val; current_val as usize]);
                            });
                        },
                        "PUSH"
                    }

                    button {
                        class: "btn btn-burst",
                        onclick: move |_| {
                            let workers = 4;
                            let per_worker = 25;
                            let start_val = stack_items.read().len() as i32;

                            // Spawn 4 independent workers
                            for w in 0..workers {
                                spawn(async move {
                                    for i in 1..=per_worker {
                                        // Each worker calculates its own range of values
                                        let current_val = start_val + (w * per_worker) + i;

                                        // 1. ATOMIC PUSH: The engine handles the thread-safety
                                        MAGIC_DATA.push(LinkedStackNode::new(current_val));

                                        // 2. THE UPDATER: Every worker fights to set the UI state
                                        // You will see the counter "flicker" between ranges 
                                        // as different workers hit their set() calls.
                                        stack_items.set(vec![current_val; current_val as usize]);

                                        // 3. YIELD: Necessary to keep the workers interleaving
                                        gloo_timers::future::TimeoutFuture::new(0).await;
                                    }
                                });
                            }
                        },
                        "BURST x100"
                    }
                    button { class: "btn btn-dec", onclick: handle_pop, "POP" }

                    button {
                        class: "btn btn-reset",
                        onclick: move |_| {
                            spawn(async move {
                                let mut dropped = 0;
                                // Pop until empty, but yield every 50 items to keep the site responsive
                                while let Ok(Some(_)) = MAGIC_DATA.pop() {
                                    dropped += 1;
                                    
                                    if dropped % 50 == 0 {
                                        stack_items.set(vec![0; (dropped % 10) as usize]); // Fun visual jitter
                                        gloo_timers::future::TimeoutFuture::new(1).await;
                                    }
                                }
                                stack_items.set(vec![]);
                                println!("Atomic Drain Complete: {} nodes reclaimed.", dropped);
                            });
                        },
                        "DRAIN ALL"
                    }
                }

                div { class: "stack-visualizer",
                    for (i, val) in stack_items.read().iter().enumerate() {
                        div {
                            class: "stack-node",
                            key: "{val}-{i}",
                            onclick: handle_pop,
                            span { "{val}" }
                        }
                    }
                }
            }
        }
    }
}
#[component]
pub fn Hero() -> Element {
    rsx! {
        header { id: "hero",
            div { class: "hero-content",
                h1 { "Alfitra Heydar Achsan" }
                div { class: "hero-subtitle",
                    p { "Software Engineer | Sidoarjo, Indonesia" }
                }
                div { class: "experience-timeline",
                    div { class: "exp-item",
                        span { class: "exp-date", "OCT 2025--PRESENT" }
                        span { class: "exp-role", "Software Engineer as Independent Contractor" }
                    }
                    div { class: "exp-item",
                        span { class: "exp-date", "JUL--OCT 2025" }
                        span { class: "exp-role", "Software Engineer @ Feroworks" }
                    }
                    div { class: "exp-item",
                        span { class: "exp-date", "JUN--JUL 2025" }
                        span { class: "exp-role", "Flutter Developer" }
                    }
                }
                div { id: "links",
                    a {
                        href: "https://github.com/achsanalfitra",
                        class: "link-chip",
                        target: "_blank",
                        rel: "noopener",
                        "GITHUB"
                    }
                    a {
                        href: "https://www.linkedin.com/in/alfitra-achsan-025a3019a/",
                        class: "link-chip",
                        target: "_blank",
                        rel: "noopener",
                        "LINKEDIN"
                    }
                }
            }
        }
    }
}

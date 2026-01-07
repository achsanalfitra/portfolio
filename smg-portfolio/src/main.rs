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

    let mut handle_pop = move |_| {
        if let Ok(Some(_)) = MAGIC_DATA.pop() {
            let new_len = stack_items.read().len().saturating_sub(1);
            if new_len == 0 {
                stack_items.set(vec![]);
            } else {
                stack_items.set(vec![new_len as i32; new_len]);
            }
        }
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
                            let current_val = stack_items.read().len() as i32 + 1;
                            MAGIC_DATA.push(LinkedStackNode::new(current_val));
                            stack_items.set(vec![current_val; current_val as usize]);
                        },
                        "PUSH"
                    }

                    // THE BURST BUTTON
                    button {
                        class: "btn btn-burst",
                        onclick: move |_| {
                            let start_val = stack_items.read().len() as i32;
                            let final_val = start_val + 100;

                            // Atomic stress test: 100 pushes in a tight loop
                            for i in (start_val + 1)..=final_val {
                                MAGIC_DATA.push(LinkedStackNode::new(i));
                            }

                            // Massive UI sync
                            stack_items.set(vec![final_val; final_val as usize]);
                        },
                        "BURST x100"
                    }

                    button { class: "btn btn-dec", onclick: handle_pop, "POP" }

                    button {
                        class: "btn btn-reset",
                        onclick: move |_| {
                            // 1. Physically drain the Atomic Stack in memory
                            // This is where rsmg-core does the heavy lifting
                            let mut dropped = 0;
                            while let Ok(Some(_)) = MAGIC_DATA.pop() {
                                dropped += 1;
                            }

                            // 2. Wipe the UI snapshot
                            stack_items.set(vec![]);

                            // Log it to the console to prove the "silent" work
                            println!("Atomic Drain Complete: {} nodes reclaimed.", dropped);
                        },
                        "DRAIN ALL"
                    }
                }

                div { class: "stack-visualizer",
                    for (i , val) in stack_items.read().iter().enumerate() {
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

use dioxus::prelude::*;
use rsmg_core::prim::stack::{LinkedStack, LinkedStackNode};
use rsmg_core::prim::array::ContiguousArray;
use std::sync::LazyLock;

// LinkedStack singleton
static MAGIC_DATA: LazyLock<LinkedStack<i32>> = LazyLock::new(LinkedStack::new);

// ContiguousArray singleton - the new hotness
static CONTIGUOUS_ARRAY: LazyLock<ContiguousArray<i32>> = LazyLock::new(ContiguousArray::new);

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut stack_items = use_signal(|| vec![]);
    let mut array_items = use_signal(|| vec![]);
    let mut race_log = use_signal(|| vec![]);

    let handle_pop = move |_| {
        spawn(async move {
            if let Ok(Some(_)) = MAGIC_DATA.pop() {
                let new_len = stack_items.read().len().saturating_sub(1);
                
                if new_len == 0 {
                    stack_items.set(vec![]);
                } else {
                    stack_items.set(vec![new_len as i32; new_len]);
                }
            }
        });
    };

    let array_snapshot = array_items.read().clone();

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Hero {}

        main { class: "container",


            // ═══════════════════════════════════════════════════════
            // CONTIGUOUSARRAY DEMO - True Concurrency First
            // ═══════════════════════════════════════════════════════
            div { id: "array-area",
                div { class: "engine-spec",
                    div { class: "spec-line",
                        span { class: "spec-label", "ENGINE:" }
                        span { class: "spec-value", "rsmg_core::prim::array::ContiguousArray" }
                        a {
                            class: "small-link-chip",
                            href: "https://crates.io/crates/rsmg_core",
                            target: "_blank",
                            "crates.io"
                        }
                    }
                    p { class: "spec-description",
                        "A "
                        strong { "true concurrency-first primitive" }
                        " that allows "
                        strong { "simultaneous index-based access" }
                        " and "
                        strong { "element-level mutations" }
                        " across all indices. Multiple workers can operate on different indices concurrently without blocking or corruption."
                    }
                }

                div { class: "controls",
                    // CONCURRENT MATRIX TRANSFORM - The star of the show
                    button {
                        class: "btn btn-burst",
                        onclick: move |_| {
                            let len = CONTIGUOUS_ARRAY.len();
                            if len == 0 {
                                let mut log = race_log.read().clone();
                                log.push("SEED first to create a matrix!".to_string());
                                if log.len() > 12 {
                                    log.remove(0);
                                }
                                race_log.set(log);
                                return;
                            }

                            // Launch 4 workers (reduced from 8 for WASM memory)
                            // Each worker performs different transformations on their section

                            let mut log = race_log.read().clone();
                            log.push(format!("Launching 4 concurrent workers on {} indices...", len));
                            if log.len() > 12 {
                                log.remove(0);
                            }
                            race_log.set(log);
                            let workers = 4; // Reduced from 8
                            for worker_id in 0..workers {
                                spawn(async move {
                                    let section_size = len / workers;
                                    let start_idx = worker_id * section_size;
                                    let end_idx = if worker_id == workers - 1 {
                                        len
                                    } else {
                                        (worker_id + 1) * section_size
                                    };
                                    // Reduced iterations from 50 to 15 for WASM
                                    for iteration in 0..15 {
                                        for idx in start_idx..end_idx {
                                            CONTIGUOUS_ARRAY
                                                .inspect_element(
                                                    idx,
                                                    |val| {
                                                        match worker_id % 4 {
                                                            0 => *val += 1,
                                                            1 => *val = (*val + 7) % 100,
                                                            2 => if iteration % 2 == 0 { *val += 2 } else { *val -= 1 }
                                                            _ => *val = (*val + idx as i32) % 100,
                                                        }
                                                    },
                                                );
                                        }
                                        // Update every 3 iterations instead of 5
                                        if iteration % 3 == 0 {
                                            let mut snapshot = vec![];
                                            for i in 0..CONTIGUOUS_ARRAY.len() {
                                                let cell = std::cell::Cell::new(0);
                                                CONTIGUOUS_ARRAY.inspect_element(i, |v| cell.set(*v));
                                                snapshot.push(cell.get());
                                            }
                                            array_items.set(snapshot);
                                            gloo_timers::future::TimeoutFuture::new(10).await;
                                        } else {
                                            gloo_timers::future::TimeoutFuture::new(5).await;
                                        }
                                    }
                                    let mut snapshot = vec![];
                                    for i in 0..CONTIGUOUS_ARRAY.len() {
                                        let cell = std::cell::Cell::new(0);
                                        CONTIGUOUS_ARRAY.inspect_element(i, |v| cell.set(*v));
                                        snapshot.push(cell.get());
                                    }
                                    array_items.set(snapshot);
                                    let mut log = race_log.read().clone();
                                    log.push(
                                        format!(
                                            "Worker {} completed {} transformations",
                                            worker_id,
                                            section_size,
                                        ),
                                    );
                                    if log.len() > 12 {
                                        log.remove(0);
                                    }
                                    race_log.set(log);
                                });
                            }
                        },
                        "MATRIX TRANSFORM (4 workers)"
                    }

                    // CONCURRENT PIPELINE - Producers + Transformers + Consumers
                    button {
                        class: "btn btn-race",
                        onclick: move |_| {
                            let mut log = race_log.read().clone();
                            log.push(
                                "Starting concurrent pipeline: PRODUCER → TRANSFORMER → CONSUMER"
                                    .to_string(),
                            );
                            if log.len() > 12 {
                                log.remove(0);
                            }
                            race_log.set(log);

                            // Producer workers: reduced to 2 producers, 10 elements each
                            for producer_id in 0..2 {
                                spawn(async move {
                                    for i in 0..10 {
                                        let value = (producer_id * 10 + i) as i32;
                                        CONTIGUOUS_ARRAY.push(value);
                                        if i % 3 == 0 {
                                            let mut snapshot = vec![];
                                            for j in 0..CONTIGUOUS_ARRAY.len() {
                                                let cell = std::cell::Cell::new(0);
                                                CONTIGUOUS_ARRAY.inspect_element(j, |v| cell.set(*v));
                                                snapshot.push(cell.get());
                                            }
                                            array_items.set(snapshot);
                                        }
                                        gloo_timers::future::TimeoutFuture::new(20).await;
                                    }
                                    let mut log = race_log.read().clone();

                                    // Transformer worker with limited iterations

                                    // Consumer worker with limited iterations
                                    log.push(format!("Producer {} finished (10 elements)", producer_id));
                                    if log.len() > 12 {
                                        log.remove(0);
                                    }
                                    race_log.set(log);
                                });
                            }
                            spawn(async move {
                                let mut iterations = 0;
                                while iterations < 50 {
                                    let len = CONTIGUOUS_ARRAY.len();
                                    if len == 0 {
                                        gloo_timers::future::TimeoutFuture::new(50).await;
                                        iterations += 1;
                                        continue;
                                    }
                                    for _ in 0..5 {
                                        if len > 0 {
                                            let idx = (len / 2) % len;
                                            CONTIGUOUS_ARRAY
                                                .inspect_element(
                                                    idx,
                                                    |val| {
                                                        *val = (*val * 3 + 7) % 500;
                                                    },
                                                );
                                        }
                                        gloo_timers::future::TimeoutFuture::new(25).await;
                                    }
                                    let mut snapshot = vec![];
                                    for i in 0..CONTIGUOUS_ARRAY.len() {
                                        let cell = std::cell::Cell::new(0);
                                        CONTIGUOUS_ARRAY.inspect_element(i, |v| cell.set(*v));
                                        snapshot.push(cell.get());
                                    }
                                    array_items.set(snapshot);
                                    iterations += 1;
                                }
                            });
                            spawn(async move {
                                let mut consumed = 0;
                                let mut iterations = 0;
                                while iterations < 30 {
                                    if let Ok(Some(_val)) = CONTIGUOUS_ARRAY.pop() {
                                        consumed += 1;
                                        let mut snapshot = vec![];
                                        for i in 0..CONTIGUOUS_ARRAY.len() {
                                            let cell = std::cell::Cell::new(0);
                                            CONTIGUOUS_ARRAY.inspect_element(i, |v| cell.set(*v));
                                            snapshot.push(cell.get());
                                        }
                                        array_items.set(snapshot);
                                        if consumed % 5 == 0 {
                                            let mut log = race_log.read().clone();
                                            log.push(format!("Consumer: {} elements consumed", consumed));
                                            if log.len() > 12 {
                                                log.remove(0);
                                            }
                                            race_log.set(log);
                                        }
                                    }
                                    gloo_timers::future::TimeoutFuture::new(50).await;
                                    iterations += 1;
                                }
                            });
                        },
                        "CONCURRENT PIPELINE"
                    }

                    // CONCURRENT STATISTICS - Calculate stats on different sections simultaneously
                    button {
                        class: "btn btn-inc",
                        onclick: move |_| {
                            let len = CONTIGUOUS_ARRAY.len();
                            if len == 0 {
                                let mut log = race_log.read().clone();
                                log.push("SEED first!".to_string());
                                if log.len() > 12 {
                                    log.remove(0);
                                }
                                race_log.set(log);
                                return;
                            }

                            let mut log = race_log.read().clone();

                            // Each worker calculates different stats on their section

                            // Calculate statistics for this section

                            // Small delay to show concurrent processing

                            // Report results

                            // Update visualization periodically

                            // Also calculate overall statistics

                            log.push("Calculating statistics on 4 sections concurrently...".to_string());
                            if log.len() > 12 {
                                log.remove(0);
                            }
                            race_log.set(log);
                            let sections = 4;
                            let section_size = len / sections;
                            for worker_id in 0..sections {
                                spawn(async move {
                                    let start_idx = worker_id * section_size;
                                    let end_idx = if worker_id == sections - 1 {
                                        len
                                    } else {
                                        (worker_id + 1) * section_size
                                    };
                                    let mut sum = 0i64;
                                    let mut count = 0;
                                    let mut max_val = i32::MIN;
                                    let mut min_val = i32::MAX;
                                    for idx in start_idx..end_idx {
                                        let cell = std::cell::Cell::new(0);
                                        CONTIGUOUS_ARRAY.inspect_element(idx, |v| cell.set(*v));
                                        let val = cell.get();
                                        sum += val as i64;
                                        count += 1;
                                        if val > max_val {
                                            max_val = val;
                                        }
                                        if val < min_val {
                                            min_val = val;
                                        }
                                        if idx % 4 == 0 {
                                            gloo_timers::future::TimeoutFuture::new(5).await;
                                        }
                                    }
                                    let avg = if count > 0 { sum / count as i64 } else { 0 };
                                    let mut log = race_log.read().clone();
                                    log.push(
                                        format!(
                                            "Section {} [{}..{}]: sum={}, avg={}, min={}, max={}",
                                            worker_id,
                                            start_idx,
                                            end_idx - 1,
                                            sum,
                                            avg,
                                            min_val,
                                            max_val,
                                        ),
                                    );
                                    if log.len() > 12 {
                                        log.remove(0);
                                    }
                                    race_log.set(log);
                                    let mut snapshot = vec![];
                                    for i in 0..CONTIGUOUS_ARRAY.len() {
                                        let cell = std::cell::Cell::new(0);
                                        CONTIGUOUS_ARRAY.inspect_element(i, |v| cell.set(*v));
                                        snapshot.push(cell.get());
                                    }
                                    array_items.set(snapshot);
                                });
                            }
                            spawn(async move {
                                gloo_timers::future::TimeoutFuture::new(100).await;
                                let mut total_sum = 0i64;
                                let mut total_count = 0;
                                let mut global_max = i32::MIN;
                                let mut global_min = i32::MAX;
                                for idx in 0..CONTIGUOUS_ARRAY.len() {
                                    let cell = std::cell::Cell::new(0);
                                    CONTIGUOUS_ARRAY.inspect_element(idx, |v| cell.set(*v));
                                    let val = cell.get();
                                    total_sum += val as i64;
                                    total_count += 1;
                                    if val > global_max {
                                        global_max = val;
                                    }
                                    if val < global_min {
                                        global_min = val;
                                    }
                                }
                                let global_avg = if total_count > 0 {
                                    total_sum / total_count as i64
                                } else {
                                    0
                                };
                                let mut log = race_log.read().clone();
                                log.push(
                                    format!(
                                        "GLOBAL: sum={}, avg={}, min={}, max={}",
                                        total_sum,
                                        global_avg,
                                        global_min,
                                        global_max,
                                    ),
                                );
                                if log.len() > 12 {
                                    log.remove(0);
                                }
                                race_log.set(log);
                            });
                        },
                        "CONCURRENT STATISTICS"
                    }

                    // SEED - Create initial matrix (reduced to 32 elements)
                    button {
                        class: "btn btn-seed",
                        onclick: move |_| {
                            spawn(async move {
                                // Clear first
                                while let Ok(Some(_)) = CONTIGUOUS_ARRAY.pop() {}

                                // Seed with a smaller pattern (32 instead of 64)
                                for i in 0..32 {
                                    CONTIGUOUS_ARRAY.push(i * 5);
                                }

                                let mut snapshot = vec![];
                                for i in 0..CONTIGUOUS_ARRAY.len() {
                                    let cell = std::cell::Cell::new(0);
                                    CONTIGUOUS_ARRAY.inspect_element(i, |v| cell.set(*v));
                                    snapshot.push(cell.get());
                                }
                                array_items.set(snapshot);

                                let mut log = race_log.read().clone();
                                log.push("Seeded 32-element matrix: [0, 5, 10, 15, ...]".to_string());
                                if log.len() > 12 {
                                    log.remove(0);
                                }
                                race_log.set(log);
                            });
                        },
                        "SEED MATRIX (32 elements)"
                    }

                    // RESET
                    button {
                        class: "btn btn-reset",
                        onclick: move |_| {
                            spawn(async move {
                                let mut count = 0;
                                while let Ok(Some(_val)) = CONTIGUOUS_ARRAY.pop() {
                                    count += 1;
                                    if count % 10 == 0 {
                                        gloo_timers::future::TimeoutFuture::new(10).await;
                                    }
                                }
                                array_items.set(vec![]);

                                let mut log = race_log.read().clone();
                                log.push(format!("Drained {} elements", count));
                                if log.len() > 12 {
                                    log.remove(0);
                                }
                                race_log.set(log);
                            });
                        },
                        "RESET"
                    }
                }

                // Array Visualizer - Shows live concurrent operations
                div { class: "array-visualizer",
                    div { class: "array-header",
                        span { "Matrix Size: {array_snapshot.len()} elements" }
                        span { " | Click any cell to mutate individually" }
                    }
                    div { class: "array-grid",
                        for idx in 0..array_snapshot.len() {
                            ArrayCell {
                                idx,
                                val: array_snapshot.get(idx).copied().unwrap_or(0),
                                array_items,
                                race_log,
                            }
                        }
                    }
                    if array_snapshot.is_empty() {
                        div { class: "array-empty", "Matrix is empty. Click 'SEED MATRIX' to begin!" }
                    }
                }

                // Activity Log - Shows concurrent operation results in real-time
                div { class: "race-log",
                    div { class: "log-header", "Live Operation Log" }
                    for (i , entry) in race_log.read().iter().enumerate() {
                        div { class: "log-entry", key: "{i}", "{entry}" }
                    }
                    if race_log.read().is_empty() {
                        div { class: "log-entry log-empty",
                            "No operations yet. Try the concurrent demos above!"
                        }
                    }
                }

                // Explanation Box
                div { class: "demo-explanation",
                    h4 { "True Concurrency First Design" }
                    ul {
                        li {
                            strong { "MATRIX TRANSFORM: " }
                            "4 workers hitting different sections of the matrix at once. "
                            "Each worker runs its own transformation logic "
                            strong { "simultaneously without ever blocking." }
                        }
                        li {
                            strong { "CONCURRENT PIPELINE: " }
                            "2 producers pushing, 1 transformer mutating, and 1 consumer popping—"
                            strong { "all at the same time." }
                            " It’s a literal demonstration of lock-free producer-consumer concurrency."
                        }
                        li {
                            strong { "CONCURRENT STATISTICS: " }
                            "4 workers crunching stats (sum, avg, min, max) across different chunks simultaneously. "
                            "A final worker aggregates the global state, "
                            "proving the array handles concurrent reads without breaking a sweat."
                        }
                        li {
                            strong { "CLICK CELLS: " }
                            "Try clicking cells while the workers are running. "
                            "Your manual mutations won't corrupt a thing because every index is independently accessible."
                        }
                    }
                }
            }

            // ═══════════════════════════════════════════════════════
            // LINKEDSTACK DEMO (existing)
            // ═══════════════════════════════════════════════════════
            div { id: "magic-area",
                div { class: "engine-spec",
                    div { class: "spec-line",
                        span { class: "spec-label", "ENGINE:" }
                        span { class: "spec-value", "rsmg_core::prim::stack::LinkedStack" }
                        a {
                            class: "small-link-chip",
                            href: "https://crates.io/crates/rsmg_core",
                            target: "_blank",
                            "crates.io"
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
                                let current_val = stack_items.read().len() as i32 + 1;
                                MAGIC_DATA.push(LinkedStackNode::new(current_val));
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

                            for w in 0..workers {
                                spawn(async move {
                                    for i in 1..=per_worker {
                                        let current_val = start_val + (w * per_worker) + i;
                                        MAGIC_DATA.push(LinkedStackNode::new(current_val));
                                        stack_items.set(vec![current_val; current_val as usize]);
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
                                while let Ok(Some(_)) = MAGIC_DATA.pop() {
                                    dropped += 1;
                                    if dropped % 50 == 0 {
                                        stack_items.set(vec![0; (dropped % 10) as usize]);
                                        gloo_timers::future::TimeoutFuture::new(1).await;
                                    }
                                }
                                stack_items.set(vec![]);
                            });
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
            } // End of interactive/main window
        }

        // Footer moved outside interactive window/main container
        footer {
            class: "site-footer",
            style: "
                margin-top: 3.5rem;
                padding: 2.2rem 0 0.9rem 0;
                text-align: center;
                font-size: 1.04em;
                background: #f5f7fa;
                color: #444b5d;
                border-top: 1px solid #e3eaf2;
                opacity: 0.93;",
            div { style: "margin-bottom: 0.4em; font-weight: 600; letter-spacing: 0.05em;",
                a {
                    href: "https://achsanalfitra.github.io/portfolio/",
                    style: "color: var(--accent); text-decoration: none;",
                    "smg-portfolio"
                }
                " © 2026"
            }
            div { style: "font-size:0.95em;",
                "by "
                a {
                    href: "https://github.com/achsanalfitra",
                    style: "color: var(--accent); text-decoration: none; font-weight: 500;",
                    "Alfitra Heydar Achsan"
                }
                " · Licensed under "
                a {
                    href: "https://creativecommons.org/licenses/by-nc/4.0/",
                    style: "color: var(--accent); text-decoration: underline dotted;",
                    "CC BY-NC 4.0"
                }
            }
        }
    }
}



#[component]
fn ArrayCell(idx: usize, val: i32, array_items: Signal<Vec<i32>>, race_log: Signal<Vec<String>>) -> Element {
    rsx! {
        div {
            class: "array-cell",
            key: "{idx}",
            onclick: move |_| {
                let idx = idx;
                spawn(async move {
                    // Direct index mutation while other operations may be running
                    CONTIGUOUS_ARRAY
                        .inspect_element(
                            idx,
                            |v| {
                                *v = (*v + 1) % 1000;
                            },
                        );
                    // Update snapshot
                    let mut snapshot = vec![];
                    for i in 0..CONTIGUOUS_ARRAY.len() {
                        let cell = std::cell::Cell::new(0);
                        CONTIGUOUS_ARRAY.inspect_element(i, |v| cell.set(*v));
                        snapshot.push(cell.get());
                    }
                    let new_val = snapshot.get(idx).copied().unwrap_or(0);
                    array_items.set(snapshot);
                    let mut log = race_log.read().clone();
                    log.push(format!("User clicked index[{}] → {}", idx, new_val));
                    if log.len() > 12 {
                        log.remove(0);
                    }
                    race_log.set(log);
                });
            },
            div { class: "cell-index", "[{idx}]" }
            div { class: "cell-value", "{val}" }
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
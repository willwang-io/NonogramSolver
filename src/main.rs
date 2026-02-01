use dioxus::prelude::*;

mod components;

use components::puzzle_viewer::PuzzleViewer;
use nonogram_solver::nonogram_solver::solve_puzzle_steps;
use nonogram_solver::puzzle_crawler::{fetch_color_puzzle, PuzzleData};

const STYLE_CSS: Asset = asset!("/assets/style.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut input_url = use_signal(String::new);

    let puzzle = use_resource(move || {
        let url = input_url();
        async move {
            if url.trim().is_empty() {
                return Ok(None);
            }
            let puzzle_id = puzzle_id_from_input(&url)
                .ok_or_else(|| "Invalid nonogram URL or ID".to_string())?;
            let data = fetch_puzzle(puzzle_id)
                .await
                .map_err(|err| err.to_string())?;
            solve_puzzle_steps(data)
                .map(Some)
                .map_err(|err| err.to_string())
        }
    });

    rsx! {
        document::Stylesheet { href: STYLE_CSS }
        div { class: "page",
            div { class: "card",
                h1 { class: "title", "Nonogram Solver" }
                div { class: "input-row",
                    input {
                        class: "input",
                        r#type: "text",
                        value: input_url,
                        placeholder: "Paste a nonograms.org URL or puzzle ID",
                        oninput: move |e| *input_url.write() = e.value(),
                    }
                }
                div { class: "hint", "Example: https://www.nonograms.org/nonograms2/i/65831 or 65831" }
                {match puzzle() {
                    None => rsx! { div { class: "status", "Loading puzzle..." } },
                    Some(Err(err)) => rsx! { div { class: "status", "Failed to load puzzle: {err}" } },
                    Some(Ok(None)) => rsx! { div { class: "status", "Enter a nonograms.org URL or ID" } },
                    Some(Ok(Some(steps))) => rsx! { div { class: "grid-wrap", PuzzleViewer { steps } } },
                }}
            }
        }
    }
}

#[get("/api/puzzle/:puzzle_id")]
async fn fetch_puzzle(puzzle_id: String) -> Result<PuzzleData, ServerFnError> {
    fetch_color_puzzle(&puzzle_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

fn puzzle_id_from_input(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.chars().all(|c| c.is_ascii_digit()) && !trimmed.is_empty() {
        return Some(trimmed.to_string());
    }
    let marker = "/i/";
    let start = trimmed.find(marker)? + marker.len();
    let rest = &trimmed[start..];
    let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

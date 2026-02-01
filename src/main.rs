use dioxus::prelude::*;

mod components;

use components::puzzle_viewer::PuzzleViewer;
use nonogram_solver::nonogram_solver::solve_puzzle_steps;
use nonogram_solver::puzzle_crawler::{fetch_puzzle as fetch_remote_puzzle, PuzzleData, PuzzleKind};

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
            let (kind, puzzle_id) = puzzle_id_from_input(&url)
                .ok_or_else(|| "Invalid nonogram URL or ID".to_string())?;
            let data = fetch_puzzle_data(puzzle_kind_param(kind), puzzle_id)
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
                div { class: "hint", "Example: https://www.nonograms.org/nonograms/i/1822 or https://www.nonograms.org/nonograms2/i/79575" }
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

#[get("/api/puzzle/:kind/:puzzle_id")]
async fn fetch_puzzle_data(kind: String, puzzle_id: String) -> Result<PuzzleData, ServerFnError> {
    let kind = match kind.as_str() {
        "bw" => PuzzleKind::BlackWhite,
        _ => PuzzleKind::Color,
    };
    fetch_remote_puzzle(kind, &puzzle_id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}

fn puzzle_id_from_input(input: &str) -> Option<(PuzzleKind, String)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("bw:") {
        let id: String = rest.chars().filter(|c| c.is_ascii_digit()).collect();
        return if id.is_empty() {
            None
        } else {
            Some((PuzzleKind::BlackWhite, id))
        };
    }
    if lower.contains("/nonograms2/") {
        return extract_id_with_kind(trimmed, PuzzleKind::Color);
    }
    if lower.contains("/nonograms/") {
        return extract_id_with_kind(trimmed, PuzzleKind::BlackWhite);
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Some((PuzzleKind::Color, trimmed.to_string()));
    }
    None
}

fn extract_id_with_kind(input: &str, kind: PuzzleKind) -> Option<(PuzzleKind, String)> {
    let marker = "/i/";
    let start = input.find(marker)? + marker.len();
    let rest = &input[start..];
    let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if id.is_empty() {
        None
    } else {
        Some((kind, id))
    }
}

fn puzzle_kind_param(kind: PuzzleKind) -> String {
    match kind {
        PuzzleKind::BlackWhite => "bw".to_string(),
        PuzzleKind::Color => "color".to_string(),
    }
}

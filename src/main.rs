use dioxus::prelude::*;

use nonogram_solver::nonogram_solver::{mask_to_color_index, solve_puzzle, SolvedPuzzle};
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
            solve_puzzle(data)
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
                div { class: "hint", "Example: https://www.nonograms.org/nonograms2/i/79559 or 79559" }
                {match puzzle() {
                    None => rsx! { div { class: "status", "Loading puzzle..." } },
                    Some(Err(err)) => rsx! { div { class: "status", "Failed to load puzzle: {err}" } },
                    Some(Ok(None)) => rsx! { div { class: "status", "Enter a nonograms.org URL or ID" } },
                    Some(Ok(Some(solution))) => rsx! { div { class: "grid-wrap", PuzzleGrid { solution } } },
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

#[component]
fn PuzzleGrid(solution: SolvedPuzzle) -> Element {
    let SolvedPuzzle { color_panel, grid } = solution;
    let rows = grid.len();
    let cols = grid.first().map(|row| row.len()).unwrap_or(0);
    let cell_size = cell_size_for_grid(rows, cols);
    let grid_style = format!(
        "display: grid; grid-template-columns: repeat({}, {}px); gap: 0;",
        cols, cell_size
    );
    let cells: Vec<String> = grid
        .iter()
        .flat_map(|row| row.iter())
        .map(|mask| {
            let color = mask_to_color_index(*mask)
                .and_then(|idx| color_panel.get(idx))
                .map(|c| c.as_str())
                .unwrap_or("#cccccc");
            format!(
                "width: {}px; height: {}px; background-color: {};",
                cell_size, cell_size, color
            )
        })
        .collect();

    rsx! {
        div { class: "grid", style: grid_style,
            for cell_style in cells {
                div { class: "cell", style: cell_style }
            }
        }
    }
}

fn cell_size_for_grid(rows: usize, cols: usize) -> usize {
    let max_dim = rows.max(cols);
    match max_dim {
        0..=10 => 32,
        11..=15 => 28,
        16..=20 => 24,
        21..=30 => 20,
        31..=40 => 16,
        41..=60 => 14,
        _ => 12,
    }
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

use dioxus::prelude::*;

use nonogram_solver::nonogram_solver::{mask_to_color_index, solve_puzzle_steps, SolveSteps};
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
                div { class: "hint", "Example: https://www.nonograms.org/nonograms2/i/79559 or 79559" }
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

#[component]
fn PuzzleViewer(steps: SolveSteps) -> Element {
    let total_steps = steps.steps.len();
    let mut current_step = use_signal(|| 0usize);
    let mut last_len = use_signal(|| 0usize);
    let steps_len = steps.steps.len();
    use_effect(move || {
        if last_len() != steps_len {
            *current_step.write() = 0;
            *last_len.write() = steps_len;
        }
    });

    let step_idx = current_step().min(steps_len.saturating_sub(1));
    let grid = steps.steps.get(step_idx).cloned().unwrap_or_default();
    let color_panel = steps.color_panel.clone();
    let is_initial = step_idx == 0;
    let max_step = total_steps.saturating_sub(1);

    rsx! {
        PuzzleGrid { color_panel, grid, is_initial }
        div { class: "step-controls",
            input {
                class: "step-slider",
                r#type: "range",
                min: "0",
                max: "{max_step}",
                value: "{step_idx}",
                oninput: move |e| {
                    if let Ok(value) = e.value().parse::<usize>() {
                        *current_step.write() = value.min(max_step);
                    }
                }
            }
            div { class: "step-label", "{step_idx} / {max_step}" }
        }
    }
}

#[component]
fn PuzzleGrid(color_panel: Vec<String>, grid: Vec<Vec<u64>>, is_initial: bool) -> Element {
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
            let color = if is_initial {
                "#ffffff"
            } else {
                mask_to_color_index(*mask)
                    .and_then(|idx| color_panel.get(idx))
                    .map(|c| c.as_str())
                    .unwrap_or("#ffffff")
            };
            format!(
                "width: {}px; height: {}px; background-color: {};",
                cell_size, cell_size, color
            )
        })
        .collect();

    let swatches: Vec<(String, String)> = color_panel
        .iter()
        .map(|color| {
            (
                format!("background-color: {};", color),
                color.to_string(),
            )
        })
        .collect();

    rsx! {
        div { class: "puzzle-meta",
            div { class: "puzzle-meta-line",
                span { class: "puzzle-size", "{cols} × {rows}" }
                span { class: "palette-label", "colors" }
                div { class: "palette-inline",
                for (style, color) in swatches {
                    div { class: "swatch-color", style: style, title: color.clone(), "data-color": "{color}" }
                }
                }
            }
        }
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
        61..=80 => 12,
        81..=100 => 10,
        _ => 8,
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

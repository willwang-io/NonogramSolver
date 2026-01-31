use dioxus::prelude::*;

use nonogram_solver::nonogram_solver::{mask_to_color_index, solve_puzzle, SolvedPuzzle};
use nonogram_solver::puzzle_crawler::{fetch_color_puzzle, PuzzleData};

const PUZZLE_ID: &str = "79550";
const CELL_SIZE_PX: usize = 24;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let puzzle = use_resource(move || async move {
        let data = fetch_puzzle(PUZZLE_ID.to_string())
            .await
            .map_err(|err| err.to_string())?;
        solve_puzzle(data).map_err(|err| err.to_string())
    });

    rsx! {
        {match puzzle() {
            None => rsx! { div { "Loading puzzle..." } },
            Some(Err(err)) => rsx! { div { "Failed to load puzzle: {err}" } },
            Some(Ok(solution)) => rsx! { PuzzleGrid { solution } },
        }}
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
    let cols = grid.first().map(|row| row.len()).unwrap_or(0);
    let grid_style = format!(
        "display: grid; grid-template-columns: repeat({}, {}px); gap: 0;",
        cols, CELL_SIZE_PX
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
                "width: {}px; height: {}px; background-color: {}; border: 1px solid #999;",
                CELL_SIZE_PX, CELL_SIZE_PX, color
            )
        })
        .collect();

    rsx! {
        div { style: grid_style,
            for cell_style in cells {
                div { style: cell_style }
            }
        }
    }
}

use dioxus::prelude::*;

use nonogram_solver::nonogram_solver::mask_to_color_index;

#[component]
pub fn PuzzleGrid(color_panel: Vec<String>, grid: Vec<Vec<u64>>, is_initial: bool) -> Element {
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
        .map(|color| (format!("background-color: {};", color), color.to_string()))
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

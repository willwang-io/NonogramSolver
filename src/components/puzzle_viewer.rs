use dioxus::prelude::*;

use crate::components::puzzle_grid::PuzzleGrid;
use nonogram_solver::nonogram_solver::SolveSteps;

#[component]
pub fn PuzzleViewer(steps: SolveSteps) -> Element {
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
            div { class: "step-label", "Steps: {step_idx} / {max_step}" }
        }
    }
}

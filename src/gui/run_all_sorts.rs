use std::rc::Rc;

use crate::{imgui_centered_text, imgui_items_to_end, imgui_spaced_separator};

use super::Gui;

pub struct RunAllSorts {
    pub length_mlt: f64, 
    pub unique_div: f64,
    pub speed: f64,
    pub distribution: i32,
    pub shuffle: i32,
    pub category: i32,
    pub visual: i32,
    pub back: bool,
}

impl RunAllSorts {
    pub fn new() -> Self {
        RunAllSorts {
            length_mlt: 1.0,
            unique_div: 2.0,
            speed: Gui::DEFAULT_SPEED,
            distribution: 0,
            shuffle: 0,
            category: 0,
            visual: 0,
            back: false,
        }
    }
}

impl Gui {
    pub fn run_all_sorts(&mut self) -> bool {
        let ui = self.imgui.new_frame();

        let mut run = false;
        self.run_all_sorts.back = false;

        ui.window("run_all_sorts")
            .position(
                [
                    self.global_x_offset, 
                    self.global_y_offset
                ], 
                imgui::Condition::Always
            )
            .size(
                [
                    self.resolution_x - self.global_x_offset * 2.0, 
                    self.resolution_y - self.global_y_offset * 2.0
                ], 
                imgui::Condition::Always
            )
            .no_decoration()
            .movable(false)
            .bg_alpha(Gui::ALPHA)
            .build(|| {
                imgui_centered_text!(ui, "Run all sorts");
                imgui_spaced_separator!(ui);
                
                ui.columns(3, "input_columns", false);

                ui.input_scalar("Length multiplier", &mut self.run_all_sorts.length_mlt).build();
                ui.input_scalar("Unique divisor", &mut self.run_all_sorts.unique_div).build();
                ui.input_scalar("Speed", &mut self.run_all_sorts.speed).build();

                ui.next_column();
                ui.next_column();
                ui.set_cursor_pos([ui.window_content_region_max()[0] - Gui::RUN_BUTTON_X_SIZE, ui.cursor_pos()[1]]);
                run = ui.button_with_size("Run!", [Gui::RUN_BUTTON_X_SIZE, Gui::RUN_BUTTON_Y_SIZE]);
                
                ui.columns(1, "separator_column", false);
                imgui_spaced_separator!(ui);

                ui.columns(4, "algo_columns", false);

                ui.text("Category");
                ui.list_box(
                    "##category", 
                    &mut self.run_all_sorts.category, 
                    &[&Rc::from("All")].into_iter()
                        .chain(self.categories.iter())
                        .map(|x| x.as_ref())
                        .collect::<Vec<&str>>(), 
                    imgui_items_to_end!(ui)
                );

                ui.set_cursor_pos([ui.cursor_pos()[0], ui.window_content_region_max()[1] - Gui::BACK_BUTTON_Y_SIZE]);
                if ui.button("Back") {
                    self.run_all_sorts.back = true;
                    run = true;
                }
                
                ui.next_column();
                ui.text("Distribution");
                ui.list_box(
                    "##distribution", 
                    &mut self.run_all_sorts.distribution, 
                    &self.distributions.iter().map(|x| x.as_ref()).collect::<Vec<&str>>(), 
                    imgui_items_to_end!(ui)
                );

                ui.next_column();
                ui.text("Shuffle");
                ui.list_box(
                    "##shuffle", 
                    &mut self.run_all_sorts.shuffle, 
                    &self.shuffles.iter().map(|x| x.as_ref()).collect::<Vec<&str>>(), 
                    imgui_items_to_end!(ui)
                );
                
                ui.next_column();
                ui.text("Visual");
                ui.list_box(
                    "##visual", 
                    &mut self.run_all_sorts.visual, 
                    &self.visuals.iter().map(|x| x.as_ref()).collect::<Vec<&str>>(), 
                    imgui_items_to_end!(ui)
                );
            });
        
        run
    }
}
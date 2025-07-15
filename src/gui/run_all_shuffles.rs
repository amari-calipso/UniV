use crate::{get_expect_mut, imgui_centered_text, imgui_items_to_end, imgui_spaced_separator};

use super::Gui;

pub struct RunAllShuffles {
    pub array_length: usize, 
    pub unique_amt: usize,
    pub speed: f64,
    pub distribution: i32,
    pub category: i32,
    pub sort: i32,
    pub visual: i32,
    pub back: bool,
}

impl RunAllShuffles {
    pub fn new() -> Self {
        RunAllShuffles {
            array_length: Gui::DEFAULT_ARRAY_LENGTH,
            unique_amt: Gui::DEFAULT_UNIQUE_AMT,
            speed: Gui::DEFAULT_SPEED,
            distribution: 0,
            category: 0,
            sort: 0,
            visual: 0,
            back: false,
        }
    }
}

impl Gui {
    pub fn run_all_shuffles(&mut self) -> bool {
        let ui = get_expect_mut!(self.imgui).new_frame();

        let mut run = false;
        self.run_all_shuffles.back = false;

        ui.window("run_all_shuffles")
            .position(
                [
                    self.global_x_offset, 
                    self.global_y_offset
                ], 
                imgui::Condition::Appearing
            )
            .size(
                [
                    self.resolution_x - self.global_x_offset * 2.0, 
                    self.resolution_y - self.global_y_offset * 2.0
                ], 
                imgui::Condition::Appearing
            )
            .no_decoration()
            .movable(false)
            .bg_alpha(Gui::ALPHA)
            .build(|| {
                imgui_centered_text!(ui, "Run all shuffles");
                imgui_spaced_separator!(ui);
                
                ui.columns(3, "input_columns", false);

                if ui.input_scalar("Array length", &mut self.run_all_shuffles.array_length).build() {
                    self.run_all_shuffles.unique_amt = self.run_all_shuffles.array_length / 2;
                }
                    
                ui.input_scalar("Unique amount", &mut self.run_all_shuffles.unique_amt).build();
                ui.input_scalar("Speed", &mut self.run_all_shuffles.speed).build();

                ui.next_column();
                ui.next_column();
                ui.set_cursor_pos([ui.window_content_region_max()[0] - Gui::RUN_BUTTON_X_SIZE, ui.cursor_pos()[1]]);
                run = ui.button_with_size("Run!", [Gui::RUN_BUTTON_X_SIZE, Gui::RUN_BUTTON_Y_SIZE]);
                
                ui.columns(1, "separator_column", false);
                imgui_spaced_separator!(ui);

                ui.columns(4, "algo_columns", false);
                
                ui.text("Distribution");
                ui.list_box(
                    "##distribution", 
                    &mut self.run_all_shuffles.distribution, 
                    &self.distributions.iter().map(|x| x.as_ref()).collect::<Vec<&str>>(), 
                    imgui_items_to_end!(ui)
                );

                ui.set_cursor_pos([ui.cursor_pos()[0], ui.window_content_region_max()[1] - Gui::BACK_BUTTON_Y_SIZE]);
                if ui.button("Back") {
                    self.run_all_shuffles.back = true;
                    run = true;
                }

                ui.next_column();
                ui.text("Category");
                if ui.list_box(
                    "##category", 
                    &mut self.run_all_shuffles.category, 
                    &self.categories.iter().map(|x| x.as_ref()).collect::<Vec<&str>>(), 
                    imgui_items_to_end!(ui)
                ) {
                    self.run_all_shuffles.sort = 0;
                }

                ui.next_column();
                ui.text("Sort");
                ui.list_box(
                    "##sort", 
                    &mut self.run_all_shuffles.sort, 
                    &self.sorts[&self.categories[self.run_all_shuffles.category as usize]]
                        .iter().map(|x| x.as_ref()).collect::<Vec<&str>>(), 
                    imgui_items_to_end!(ui)
                );
                
                ui.next_column();
                ui.text("Visual");
                ui.list_box(
                    "##visual", 
                    &mut self.run_all_shuffles.visual, 
                    &self.visuals.iter().map(|x| x.as_ref()).collect::<Vec<&str>>(), 
                    imgui_items_to_end!(ui)
                );
            });
        
        run
    }
}
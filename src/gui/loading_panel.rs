use std::sync::{atomic::{self, AtomicBool}, Arc};

use crate::get_expect_mut;

use super::Gui;

pub struct LoadingPanel {
    pub message: String,
    pub running: Arc<AtomicBool>
}

impl LoadingPanel {
    pub fn new() -> Self {
        LoadingPanel {
            message: String::from("Loading..."),
            running: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn set_message(&mut self, message: &str) {
        self.message.clear();
        self.message.push_str(message);
    }
}

impl Gui {
    pub fn loading_panel(&mut self) -> bool {
        let ui = get_expect_mut!(self.imgui).new_frame();

        let mut quit = false;

        ui.window("loading_panel")
            .position(
                [
                    self.resolution_x / 2.0 - self.small_window_size_x / 2.0, 
                    self.resolution_y / 2.0 - self.small_window_size_y / 2.0
                ], 
                imgui::Condition::Appearing
            )
            .size(
                [
                    self.small_window_size_x, 
                    self.small_window_size_y
                ], 
                imgui::Condition::Appearing
            )
            .no_decoration()
            .bg_alpha(Gui::ALPHA)
            .build(|| {
                let text_size = ui.calc_text_size(self.loading_panel.message.as_str());
                ui.set_cursor_pos([(self.small_window_size_x - text_size[0]) / 2.0, (self.small_window_size_y - text_size[1]) / 2.0]);
                ui.align_text_to_frame_padding();
                ui.text(self.loading_panel.message.as_str());

                if !self.loading_panel.running.load(atomic::Ordering::Relaxed) {
                    quit = true;
                }
            });

        quit
    }
}
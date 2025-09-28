use std::rc::Rc;

use ordered_float::OrderedFloat;

use super::Gui;

pub struct Popup {
    pub title:   String,
    pub message: String,
}

impl Popup {
    pub fn new() -> Self {
        Popup {
            title: String::from("Popup"),
            message: String::from("No message provided"),
        }
    }

    pub fn set(&mut self, title: &str, message: &str) -> Result<(), Rc<str>> {
        if title == "##message" {
            return Err(Rc::from("The popup window title cannot be '##message'"));
        }

        self.title.clear();
        self.title.push_str(title);

        self.message.clear();
        self.message.push_str(message);

        Ok(())
    }
}

impl Gui {
    pub fn popup(&mut self) -> bool {
        let ui = self.imgui.new_frame();

        let mut quit = false;

        ui.window(self.popup.title.as_str())
            .position(
                [
                    self.resolution_x / 2.0 - self.small_window_size_x / 2.0, 
                    self.resolution_y / 2.0 - self.small_window_size_y / 2.0
                ], 
                imgui::Condition::Always
            )
            .size(
                [
                    self.small_window_size_x, 
                    self.small_window_size_y
                ], 
                imgui::Condition::Always
            )
            .bg_alpha(Gui::ALPHA)
            .build(|| {
                let max = ui.window_content_region_max();

                ui.input_text_multiline(
                    "##message", &mut self.popup.message, 
                    [
                        ui.content_region_avail()[0], 
                        std::cmp::max(OrderedFloat(max[1] - Gui::OK_BUTTON_Y_SIZE * 2.0), OrderedFloat(1.0)).0
                    ]
                )
                    .read_only(true)
                    .build();

                ui.spacing();
                ui.spacing();

                ui.set_cursor_pos([max[0] / 2.0 - Gui::OK_BUTTON_X_SIZE / 2.0, max[1] - Gui::OK_BUTTON_Y_SIZE]);
                if ui.button_with_size("OK", [Gui::OK_BUTTON_X_SIZE, Gui::OK_BUTTON_Y_SIZE]) {
                    quit = true;
                }
            });

        quit
    }
}
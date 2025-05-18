use std::rc::Rc;

use super::Gui;

pub struct TextInput {
    pub title:   String,
    pub message: String,
    pub input:   String,
}

impl TextInput {
    pub fn new() -> Self {
        TextInput {
            title: String::from("Text input screen"),
            message: String::from("No message provided"),
            input: String::from(""),
        }
    }

    pub fn set(&mut self, title: &str, message: &str, input: &str) -> Result<(), Rc<str>> {
        if title == "##text_input" {
            return Err(Rc::from("The text input window title cannot be '##text_input'"));
        }

        if title == "##message" {
            return Err(Rc::from("The text input window title cannot be '##message'"));
        }

        self.title.clear();
        self.title.push_str(title);

        self.message.clear();
        self.message.push_str(message);

        self.input.clear();
        self.input.push_str(input);

        Ok(())
    }
}

impl Gui {
    pub fn text_input(&mut self) -> bool {
        let ui = self.imgui.new_frame();

        let mut quit = false;

        ui.window(self.text_input.title.as_str())
            .position(
                [
                    self.resolution_x / 2.0 - self.medium_window_size_x / 2.0, 
                    self.resolution_y / 2.0 - self.medium_window_size_y / 2.0
                ], 
                imgui::Condition::Appearing
            )
            .size(
                [
                    self.medium_window_size_x, 
                    self.medium_window_size_y
                ], 
                imgui::Condition::Appearing
            )
            .bg_alpha(Gui::ALPHA)
            .build(|| {
                ui.input_text_multiline("##message", &mut self.text_input.message, [ui.window_content_region_max()[0], 0.0])
                    .read_only(true)
                    .build();

                ui.spacing();

                ui.set_next_item_width(ui.content_region_avail()[0] - 10.0);
                ui.input_text("##text_input", &mut self.text_input.input).build();

                ui.spacing();
                ui.set_cursor_pos([ui.window_content_region_max()[0] / 2.0 - Gui::OK_BUTTON_X_SIZE / 2.0, ui.cursor_pos()[1]]);
                if ui.button_with_size("OK", [Gui::OK_BUTTON_X_SIZE, Gui::OK_BUTTON_Y_SIZE]) {
                    quit = true;
                }
            });

        quit
    }
}
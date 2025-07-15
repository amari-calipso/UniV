use std::rc::Rc;

use crate::get_expect_mut;

use super::Gui;

pub struct Selection {
    pub title:   String,
    pub message: String,
    pub items: Vec<Rc<str>>,
    pub index: usize
}

impl Selection {
    pub fn new() -> Self {
        Selection {
            title: String::from("Selection screen"),
            message: String::from("No message provided"),
            items: Vec::new(),
            index: 0
        }
    }

    pub fn set(&mut self, title: &str, message: &str, items: Vec<Rc<str>>, index: usize) -> Result<(), Rc<str>> {
        if index >= items.len() {
            return Err(format!("Invalid default index {} for length {}", index, items.len()).into());
        }

        if title == "##selection" {
            return Err(Rc::from("The window selection title cannot be '##selection'"));
        }

        self.title.clear();
        self.title.push_str(title);

        self.message.clear();
        self.message.push_str(message);

        self.items.clear();
        self.items.extend(items);

        self.index = index;

        Ok(())
    }
}

impl Gui {
    pub fn selection(&mut self) -> bool {
        let ui = get_expect_mut!(self.imgui).new_frame();

        let mut quit = false;

        ui.window(self.selection.title.as_str())
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
                ui.text(self.selection.message.as_str());
                ui.spacing();

                let max = ui.window_content_region_max();
                imgui::ListBox::new("##selection")
                    .size([max[0] - 10.0, max[1] - Gui::OK_BUTTON_Y_SIZE * 3.0])
                    .build(ui, || {
                        for (i, item) in self.selection.items.iter().enumerate() {
                            if ui.selectable_config(item).selected(i == self.selection.index).build() {
                                self.selection.index = i;
                            }
                        }
                    });

                ui.spacing();
                ui.set_cursor_pos([ui.window_content_region_max()[0] / 2.0 - Gui::OK_BUTTON_X_SIZE / 2.0, ui.cursor_pos()[1]]);
                if ui.button_with_size("OK", [Gui::OK_BUTTON_X_SIZE, Gui::OK_BUTTON_Y_SIZE]) {
                    quit = true;
                }
            });

        quit
    }
}
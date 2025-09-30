use std::rc::Rc;

use super::Gui;

pub struct Selection {
    pub title:   String,
    pub message: String,
    pub items: Vec<Rc<str>>,
    pub index: usize,

    pub enable_back: bool,
    pub back: bool,
}

impl Selection {
    pub fn new() -> Self {
        Selection {
            title: String::from("Selection screen"),
            message: String::from("No message provided"),
            items: Vec::new(),
            index: 0,
            enable_back: false,
            back: false,
        }
    }

    fn set_internal(&mut self, title: &str, message: &str, items: Vec<Rc<str>>, index: usize) -> Result<(), Rc<str>> {
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

    pub fn set(&mut self, title: &str, message: &str, items: Vec<Rc<str>>, index: usize) -> Result<(), Rc<str>> {
        self.enable_back = false;
        self.set_internal(title, message, items, index)
    }

    pub fn set_with_back(&mut self, title: &str, message: &str, items: Vec<Rc<str>>, index: usize) -> Result<(), Rc<str>> {
        self.enable_back = true;
        self.back = false;
        self.set_internal(title, message, items, index)
    }
}

impl Gui {
    pub fn selection(&mut self) -> bool {
        let ui = self.imgui.new_frame();

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

                if self.selection.enable_back {
                    ui.set_cursor_pos([ui.cursor_pos()[0], ui.window_content_region_max()[1] - Gui::BACK_BUTTON_Y_SIZE]);
                    if ui.button("Back") {
                        self.selection.back = true;
                        quit = true;
                    }

                    ui.same_line();
                    ui.set_cursor_pos([
                        ui.window_content_region_max()[0] - Gui::OK_BUTTON_X_SIZE, 
                        ui.window_content_region_max()[1] - Gui::OK_BUTTON_Y_SIZE
                    ]);
                    
                    if ui.button_with_size("OK", [Gui::OK_BUTTON_X_SIZE, Gui::OK_BUTTON_Y_SIZE]) {
                        quit = true;
                    }
                } else {
                    ui.set_cursor_pos([ui.window_content_region_max()[0] / 2.0 - Gui::OK_BUTTON_X_SIZE / 2.0, ui.cursor_pos()[1]]);
                    if ui.button_with_size("OK", [Gui::OK_BUTTON_X_SIZE, Gui::OK_BUTTON_Y_SIZE]) {
                        quit = true;
                    }
                }
            });

        quit
    }
}
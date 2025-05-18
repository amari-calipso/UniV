use std::rc::Rc;

use crate::{imgui_centered_text, imgui_spaced_separator};

use super::{AutomationFileInfo, Gui};

pub struct AutomationSelection {
    pub title: String,
    pub items: Vec<AutomationFileInfo>,
    pub index: usize,
    pub back: bool
}

impl AutomationSelection {
    pub fn new() -> Self {
        AutomationSelection {
            title: String::from("Select automation"),
            items: Vec::new(),
            index: 0,
            back: false
        }
    }

    pub fn set(&mut self, title: &str, items: Vec<AutomationFileInfo>, index: usize) -> Result<(), Rc<str>> {
        if index >= items.len() {
            return Err(format!("Invalid default index {} for length {}", index, items.len()).into());
        }

        if title == "##automation-selection" {
            return Err(Rc::from("The window selection title cannot be '##automation-selection'"));
        }

        if title == "##automation-selection-columns" {
            return Err(Rc::from("The window selection title cannot be '##automation-selection-columns'"));
        }

        if title == "##automation-selection-listbox" {
            return Err(Rc::from("The window selection title cannot be '##automation-selection-listbox'"));
        }

        if title == "##description" {
            return Err(Rc::from("The window selection title cannot be '##description'"));
        }

        self.title.clear();
        self.title.push_str(title);

        self.items.clear();
        self.items.extend(items);

        self.index = index;

        Ok(())
    }
}

impl Gui {
    pub fn automation_selection(&mut self) -> bool {
        let ui = self.imgui.new_frame();

        let mut quit = false;

        ui.window("##automation-selection")
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
                imgui_centered_text!(ui, self.automation_selection.title.as_str());
                imgui_spaced_separator!(ui);

                ui.columns(2, "##automation-selection-columns", true);

                let max = ui.window_content_region_max();
                imgui::ListBox::new("##automation-selection-listbox")
                    .size([max[0] - 10.0, max[1] - Gui::OK_BUTTON_Y_SIZE * 2.0])
                    .build(ui, || {
                        for (i, item) in self.automation_selection.items.iter().enumerate() {
                            if ui.selectable_config(&item.filename).selected(i == self.automation_selection.index).build() {
                                self.automation_selection.index = i;
                            }
                        }
                    });

                ui.set_cursor_pos([ui.cursor_pos()[0], ui.window_content_region_max()[1] - Gui::BACK_BUTTON_Y_SIZE]);
                if ui.button("Back") {
                    self.automation_selection.back = true;
                    quit = true;
                }

                ui.next_column();

                ui.input_text_multiline(
                    "##description", 
                    &mut self.automation_selection.items[self.automation_selection.index].description, 
                    [ui.window_content_region_max()[0], ui.window_content_region_max()[1] - Gui::OK_BUTTON_Y_SIZE * 2.0]
                ).read_only(true).build();

                ui.set_cursor_pos([
                    ui.window_content_region_max()[0] - Gui::OK_BUTTON_X_SIZE, 
                    ui.window_content_region_max()[1] - Gui::OK_BUTTON_Y_SIZE
                ]);

                if ui.button_with_size("OK", [Gui::OK_BUTTON_X_SIZE, Gui::OK_BUTTON_Y_SIZE]) {
                    quit = true;
                }
            });

        quit
    }
}
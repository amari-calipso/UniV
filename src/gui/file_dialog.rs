// original code from https://github.com/tseli0s/imfile
// modified to fit our needs (and to fix broken dependencies)

use std::{cmp::Ordering, fs, path::PathBuf, rc::Rc};

use crate::program_dir;

use super::{FileOption, Gui};

pub struct FileDialog {
    title:        String,
    dirs_only:    bool,
    current_dir:  PathBuf,
    pub selected: FileOption,
}

impl FileDialog {
    pub fn new() -> Self {
        Self {
            title: String::from("Open File"),
            dirs_only: false,
            selected: FileOption::None,
            current_dir: PathBuf::new()
        }
    }

    pub fn set(&mut self, title: &str, dirs_only: bool) -> Result<(), Rc<str>> {
        if title == "Path Selection" {
            return Err(Rc::from("The text input window title cannot be 'Path Selection'"));
        }

        if title == "Select file / directory" {
            return Err(Rc::from("The text input window title cannot be 'Select file / directory'"));
        }

        if title == "controls" {
            return Err(Rc::from("The text input window title cannot be 'controls'"));
        }

        self.title.clear();
        self.title.push_str(title);

        self.dirs_only = dirs_only;

        self.current_dir = {
            if let Ok(dir) = std::env::current_dir() {
                dir
            } else {
                program_dir!().clone()
            }
        };

        Ok(())
    }
}

impl Gui {
    pub fn file_dialog(&mut self) -> bool {
        let ui = self.imgui.new_frame();

        let mut quit = false;

        self.file_dialog.selected = FileOption::None;
        ui.window(&self.file_dialog.title)
            .position(
                [
                    self.resolution_x / 2.0 - self.medium_window_size_x / 2.0, 
                    self.resolution_y / 2.0 - self.medium_window_size_y / 2.0
                ], 
                imgui::Condition::Always
            )
            .size(
                [
                    self.medium_window_size_x, 
                    self.medium_window_size_y
                ], 
                imgui::Condition::Always
            )
            .build(|| {
                ui.child_window("Path Selection")
                    .horizontal_scrollbar(false)
                    .border(true)
                    .size([0.0, 32.0])
                    .build(||{
                        ui.button("Path: ");
                        ui.same_line();
                        self.file_dialog.current_dir.clone().iter().enumerate().for_each(|(i, dir)|{
                            if ui.button(dir.to_string_lossy()) {
                                let mut j = self.file_dialog.current_dir.iter().count();
                                while j > i {
                                    self.file_dialog.current_dir.pop();
                                    j -= 1;
                                }

                                self.file_dialog.current_dir.push(dir);
                            }

                            if ui.is_item_hovered() {
                                ui.tooltip_text(format!("Directory: {}", dir.to_string_lossy()));
                            }

                            ui.same_line();
                        });
                    });

                ui.child_window("Select file / directory")
                    .border(true)
                    .size([0.0, -32.0])
                    .build(|| {
                        let mut entries: Vec<_> = fs::read_dir(&self.file_dialog.current_dir)
                            .unwrap()
                            .map(|entry| entry.expect("Filesystem entry error"))
                            .collect();

                        entries.sort_by(|a, b| {
                            if a.path().is_dir() && !b.path().is_dir() {
                                Ordering::Less
                            } else if !a.path().is_dir() && b.path().is_dir() {
                                Ordering::Greater
                            } else {
                                a.path().cmp(&b.path())
                            }
                        });

                        for entry in entries {
                            if entry.path().is_file() && !self.file_dialog.dirs_only {
                                if ui.button(format!("[file]\t{}", PathBuf::from(entry.path().iter().last().unwrap()).display())) {
                                    self.file_dialog.selected = FileOption::Some(entry.path());
                                    quit = true;
                                }
                            } else if entry.path().is_dir() {
                                if ui.button(format!("[dir] \t{}", PathBuf::from(entry.path().iter().last().unwrap()).display())) {
                                    self.file_dialog.current_dir.push(entry.path());
                                }
                            }
                        }
                    });

                ui.child_window("controls")
                    .border(false)
                    .build(||{
                        ui.same_line();

                        if ui.button("Back") {
                            self.file_dialog.current_dir.pop();
                        }

                        ui.same_line();
                        ui.button("Open");
                        ui.same_line();
                        if ui.button("Cancel") {
                            self.file_dialog.selected = FileOption::Canceled;
                            quit = true;
                        }
                    });
            });

        quit
    }
}
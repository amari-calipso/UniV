use imgui::ItemHoveredFlags;

use crate::{settings::{CallStackDepthMode, StatMode}, utils::linear_search};

use super::Gui;

pub struct ConfigureStats {
    pub stat_modes: Vec<&'static str>,
    pub call_stack_depth_modes: Vec<&'static str>
}

impl ConfigureStats {
    pub fn new() -> Self {
        ConfigureStats { 
            stat_modes: StatMode::all().into_iter().map(|x| x.as_str()).collect(),
            call_stack_depth_modes: CallStackDepthMode::all().into_iter().map(|x| x.as_str()).collect()
        }
    }

    fn read_stat_option(&mut self, stat: Option<StatMode>, idx: &mut usize, enabled: &mut bool) {
        if let Some(stat) = stat {
            *idx = linear_search(&self.stat_modes, &stat.as_str()).unwrap();
            *enabled = true;
        } else {
            *enabled = false;
        }
    }

    fn write_stat_option(&self, stat: &mut Option<StatMode>, idx: usize, enabled: bool) {
        if enabled {
            *stat = Some(StatMode::from_str(self.stat_modes[idx]).unwrap());
        } else {
            *stat = None;
        }
    }

    fn read_call_stack_depth_option(&mut self, rdm: Option<CallStackDepthMode>, idx: &mut usize, enabled: &mut bool) {
        if let Some(rdm) = rdm {
            *idx = linear_search(&self.call_stack_depth_modes, &rdm.as_str()).unwrap();
            *enabled = true;
        } else {
            *enabled = false;
        }
    }

    fn write_call_stack_depth_option(&self, rdm: &mut Option<CallStackDepthMode>, idx: usize, enabled: bool) {
        if enabled {
            *rdm = Some(CallStackDepthMode::from_str(self.call_stack_depth_modes[idx]).unwrap());
        } else {
            *rdm = None;
        }
    }
}

macro_rules! stat_gui {
    ($slf: expr, $ui: expr, $name: literal, $idx: ident, $enabled: ident) => {
        {
            $ui.checkbox($name, &mut $enabled);
            $ui.same_line();
            $ui.disabled(!$enabled, || {
                $ui.combo_simple_string(concat!("##", $name), &mut $idx, &$slf.stat_modes);
            });
        }  
    };
}

macro_rules! call_stack_depth_gui {
    ($slf: expr, $ui: expr, $name: literal, $idx: ident, $enabled: ident) => {
        {
            $ui.checkbox($name, &mut $enabled);
            $ui.same_line();
            $ui.disabled(!$enabled, || {
                $ui.combo_simple_string(concat!("##", $name), &mut $idx, &$slf.call_stack_depth_modes);
            });
        }  
    };
}

impl Gui {
    pub fn configure_stats(&mut self) -> bool {
        let mut writes_enabled = true;
        let mut writes_idx = 0usize;
        let mut swaps_enabled = true;
        let mut swaps_idx = 0usize;
        let mut reads_enabled = true;
        let mut reads_idx = 0usize;
        let mut call_stack_depth_enabled = true;
        let mut call_stack_depth_idx = 0usize;
        let mut recursion_depth_enabled = true;
        let mut recursion_depth_idx = 0usize;

        self.configure_stats.read_stat_option(self.settings.object.stats.writes, &mut writes_idx, &mut writes_enabled);
        self.configure_stats.read_stat_option(self.settings.object.stats.swaps,  &mut swaps_idx,  &mut swaps_enabled);
        self.configure_stats.read_stat_option(self.settings.object.stats.reads,  &mut reads_idx,  &mut reads_enabled);

        self.configure_stats.read_call_stack_depth_option(
            self.settings.object.stats.call_stack_depth, 
            &mut call_stack_depth_idx,  
            &mut call_stack_depth_enabled
        );

        self.configure_stats.read_call_stack_depth_option(
            self.settings.object.stats.recursion_depth, 
            &mut recursion_depth_idx,  
            &mut recursion_depth_enabled
        );

        let ui = self.imgui.new_frame();

        ui.window("Configure statistics")
            .position(
                [
                    self.resolution_x / 2.0 - self.config_stats_window_size_x / 2.0, 
                    self.resolution_y / 2.0 - self.config_stats_window_size_y / 2.0
                ], 
                imgui::Condition::Appearing
            )
            .size(
                [
                    self.config_stats_window_size_x, 
                    self.config_stats_window_size_y
                ], 
                imgui::Condition::Appearing
            )
            .bg_alpha(Gui::ALPHA)
            .build(|| {
                ui.checkbox("Array length", &mut self.settings.object.stats.array_length);
                ui.spacing();
                
                ui.checkbox("Currently running", &mut self.settings.object.stats.currently_running);
                if ui.is_item_hovered() {
                    ui.tooltip_text(concat!(
                        "Shows a line of text with the name and category of\n",
                        "the currently running algorithm"
                    ));
                }

                ui.spacing();

                stat_gui!(self.configure_stats, ui, "Writes", writes_idx, writes_enabled);
                ui.spacing();

                stat_gui!(self.configure_stats, ui, "Reads", reads_idx, reads_enabled);
                ui.spacing();

                ui.checkbox("Swaps", &mut swaps_enabled);
                ui.disabled(!swaps_enabled, || {
                    ui.same_line();
                    ui.checkbox("Show failed", &mut self.settings.object.stats.failed_swaps);
                    if ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
                        ui.tooltip(|| {
                            ui.text("If unreliability is enabled, shows how many swaps failed");
                        });
                    }

                    ui.same_line();
                    ui.combo_simple_string("##swaps", &mut swaps_idx, &self.configure_stats.stat_modes);
                });
                ui.spacing();

                ui.checkbox("Comparisons", &mut self.settings.object.stats.comparisons);
                ui.disabled(!self.settings.object.stats.comparisons, || {
                    ui.same_line();
                    ui.checkbox("Show failed", &mut self.settings.object.stats.failed_comparisons);
                    if ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
                        ui.tooltip(|| {
                            ui.text("If unreliability is enabled, shows how many comparisons failed");
                        });
                    }
                });
                ui.spacing();

                ui.checkbox("Time", &mut self.settings.object.stats.time);
                ui.spacing();

                call_stack_depth_gui!(self.configure_stats, ui, "Call stack depth", call_stack_depth_idx, call_stack_depth_enabled);
                ui.spacing();

                call_stack_depth_gui!(self.configure_stats, ui, "Recursion depth", recursion_depth_idx, recursion_depth_enabled);

                ui.set_cursor_pos([ui.cursor_pos()[0], ui.window_content_region_max()[1] - Gui::BACK_BUTTON_Y_SIZE]);
                if ui.button("Back") {
                    self.build_fn = Gui::settings;
                }
            });

        self.configure_stats.write_stat_option(&mut self.settings.object.stats.writes, writes_idx, writes_enabled);
        self.configure_stats.write_stat_option(&mut self.settings.object.stats.swaps,   swaps_idx,  swaps_enabled);
        self.configure_stats.write_stat_option(&mut self.settings.object.stats.reads,   reads_idx,  reads_enabled);

        self.configure_stats.write_call_stack_depth_option(
            &mut self.settings.object.stats.call_stack_depth,   
            call_stack_depth_idx,  
            call_stack_depth_enabled
        );

        self.configure_stats.write_call_stack_depth_option(
            &mut self.settings.object.stats.recursion_depth,   
            recursion_depth_idx,  
            recursion_depth_enabled
        );

        false
    }
}
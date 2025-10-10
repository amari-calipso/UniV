use super::Gui;

const PAUSED_TEXT: &str = "Paused";

impl Gui {
    pub fn paused(&mut self) -> bool {
        let ui = self.imgui.new_frame();

        ui.window("paused")
            .position(
                [
                    self.resolution_x / 2.0 - self.paused_popup_size_x / 2.0, 
                    self.resolution_y / 2.0 - self.paused_popup_size_y / 2.0
                ], 
                imgui::Condition::Appearing
            )
            .size(
                [
                    self.paused_popup_size_x, 
                    self.paused_popup_size_y
                ], 
                imgui::Condition::Appearing
            )
            .no_decoration()
            .bg_alpha(Gui::ALPHA)
            .build(|| {
                let text_size = ui.calc_text_size(PAUSED_TEXT);
                ui.set_cursor_pos([(self.paused_popup_size_x - text_size[0]) / 2.0, (self.paused_popup_size_y - text_size[1]) / 2.0]);
                ui.align_text_to_frame_padding();
                ui.text(PAUSED_TEXT);
            });

        false
    }
}
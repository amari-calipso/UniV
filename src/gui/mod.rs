use std::{cell::OnceCell, collections::HashMap, path::PathBuf, rc::Rc};

use automation_selection::AutomationSelection;
use file_dialog::FileDialog;
use imgui::FontSource;
use loading_panel::LoadingPanel;
use popup::Popup;
use raylib::{color::Color, ffi::TraceLogLevel, prelude::{RaylibDraw, RaylibTextureModeExt}, texture::RenderTexture2D, window::{get_current_monitor, get_monitor_refresh_rate}, RaylibHandle, RaylibThread};
use run_all_shuffles::RunAllShuffles;
use run_all_sorts::RunAllSorts;
use run_sort::RunSort;
use selection::Selection;
use settings::Settings;
use text_input::TextInput;

use crate::{get_expect, get_expect_mut, univm::object::ExecutionInterrupt, LOG_LEVEL, REFERENCE_FRAMERATE};

mod run_sort;
mod selection;
mod text_input;
mod popup;
mod loading_panel;
mod file_dialog;
mod settings;
mod run_all_sorts;
mod run_all_shuffles;
mod automation_selection;

#[derive(Clone)]
pub enum FileOption {
    Some(PathBuf),
    None,
    Canceled
}

#[derive(Clone)]
pub struct AutomationFileInfo {
    pub filename:    Rc<str>,
    pub description: String
}

impl AutomationFileInfo {
    pub fn new(filename: Rc<str>, description: String) -> Self {
        AutomationFileInfo { filename, description }
    }
}

pub struct Gui {
    imgui: imgui::Context,
    renderer: OnceCell<raylib_imgui_rs::Renderer>,
    pub build_fn: fn(&mut Self) -> bool,

    pub target_fps: u32,
    pub background: OnceCell<RenderTexture2D>,

    resolution_x: f32,
    resolution_y: f32,

    global_x_offset: f32,
    global_y_offset: f32,

    small_window_size_x: f32,
    small_window_size_y: f32,
    medium_window_size_x: f32,
    medium_window_size_y: f32,
    settings_window_size_x: f32,
    settings_window_size_y: f32,

    pub distributions:    Vec<Rc<str>>,
    pub shuffles:         Vec<Rc<str>>,
    pub sorts:            HashMap<Rc<str>, Vec<Rc<str>>>,
    pub categories:       Vec<Rc<str>>,
    pub visuals:          Vec<Rc<str>>,
    pub sounds:           Vec<Rc<str>>,
    pub rotations:        Vec<Rc<str>>,
    pub pivot_selections: Vec<Rc<str>>,
    
    pub run_sort:             RunSort,
    pub selection:            Selection,
    pub text_input:           TextInput,
    pub popup:                Popup,
    pub loading_panel:        LoadingPanel,
    pub file_dialog:          FileDialog,
    pub settings:             Settings,
    pub run_all_sorts:        RunAllSorts,
    pub run_all_shuffles:     RunAllShuffles,
    pub automation_selection: AutomationSelection,
}

impl Gui {
    const ALPHA: f32 = 0.8;

    const DEFAULT_ARRAY_LENGTH: usize = 2048;
    const DEFAULT_UNIQUE_AMT: usize = Gui::DEFAULT_ARRAY_LENGTH / 2;
    const DEFAULT_SPEED: f64 = 1.0;

    const GLOBAL_OFFSET_DIV: f32 = 40.0;
    const SMALL_WINDOW_DIV_X: f32 = 3.0;
    const SMALL_WINDOW_DIV_Y: f32 = 3.5;
    const MEDIUM_WINDOW_DIV: f32 = 2.0;
    const SETTINGS_WINDOW_DIV_X: f32 = 2.0;
    const SETTINGS_WINDOW_DIV_Y: f32 = 1.15;

    const ITEMS_Y_ADJUST: f32 = 10.0;

    const RUN_BUTTON_X_SIZE: f32 = 200.0;
    const RUN_BUTTON_Y_SIZE: f32 = 70.0;

    const OK_BUTTON_X_SIZE: f32 = 100.0;
    const OK_BUTTON_Y_SIZE: f32 = 50.0;

    const SAVE_BUTTON_X_SIZE: f32 = Gui::OK_BUTTON_X_SIZE;
    const SAVE_BUTTON_Y_SIZE: f32 = Gui::OK_BUTTON_Y_SIZE;

    const BACK_BUTTON_Y_SIZE: f32 = 20.0;

    pub fn new() -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        Gui {
            imgui,
            renderer: OnceCell::new(),
            build_fn: Self::placeholder_fn,

            target_fps: REFERENCE_FRAMERATE,
            background: OnceCell::new(),

            resolution_x: 0.0,
            resolution_y: 0.0,

            global_x_offset: 0.0,
            global_y_offset: 0.0,

            small_window_size_x: 0.0,
            small_window_size_y: 0.0,
            medium_window_size_x: 0.0,
            medium_window_size_y: 0.0,
            settings_window_size_x: 0.0,
            settings_window_size_y: 0.0,

            distributions: Vec::new(),
            shuffles: Vec::new(),
            sorts: HashMap::new(),
            categories: Vec::new(),
            visuals: Vec::new(),
            sounds: Vec::new(),
            rotations: Vec::new(),
            pivot_selections: Vec::new(),

            run_sort: RunSort::new(),
            selection: Selection::new(),
            text_input: TextInput::new(),
            popup: Popup::new(),
            loading_panel: LoadingPanel::new(),
            file_dialog: FileDialog::new(),
            settings: Settings::new(),
            run_all_sorts: RunAllSorts::new(),
            run_all_shuffles: RunAllShuffles::new(),
            automation_selection: AutomationSelection::new()
        }
    }

    pub fn init(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.imgui.fonts().clear();
        self.imgui.fonts()
            .add_font(&[
                FontSource::DefaultFontData { config: None }
            ]);

        let _ = self.renderer.set(
            raylib_imgui_rs::Renderer::create(&mut self.imgui, rl, thread)
        );

        let width = rl.get_screen_width() as u32;
        let height = rl.get_screen_height() as u32;

        let mut background = rl.load_render_texture(thread, width, height)
            .expect("Could not load render texture");
        rl.begin_texture_mode(thread, &mut background).clear_background(Color::BLACK);
        
        let _ = self.background.set(background);

        self.resolution_x = width as f32;
        self.resolution_y = height as f32;

        self.global_x_offset = self.resolution_x / Gui::GLOBAL_OFFSET_DIV;
        self.global_y_offset = self.resolution_y / Gui::GLOBAL_OFFSET_DIV;
        
        self.small_window_size_x = self.resolution_x / Gui::SMALL_WINDOW_DIV_X;
        self.small_window_size_y = self.resolution_y / Gui::SMALL_WINDOW_DIV_Y;
        self.medium_window_size_x = self.resolution_x / Gui::MEDIUM_WINDOW_DIV;
        self.medium_window_size_y = self.resolution_y / Gui::MEDIUM_WINDOW_DIV;
        self.settings_window_size_x = self.resolution_x / Gui::SETTINGS_WINDOW_DIV_X;
        self.settings_window_size_y = self.resolution_y / Gui::SETTINGS_WINDOW_DIV_Y;
    }

    fn placeholder_fn(&mut self) -> bool {
        false
    }

    pub fn run(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        rl.set_trace_log(TraceLogLevel::LOG_NONE);
        rl.set_target_fps(get_monitor_refresh_rate(get_current_monitor()) as u32);
        rl.set_trace_log(LOG_LEVEL);

        let mut quit = true;

        while !rl.window_should_close() {
            get_expect_mut!(self.renderer).update(&mut self.imgui, rl);

            let done = (self.build_fn)(self);

            let mut draw = rl.begin_drawing(thread);
            draw.draw_texture(get_expect!(self.background), 0, 0, Color::WHITE);
            get_expect!(self.renderer).render(&mut self.imgui, &mut draw);

            if done {
                quit = false;
                break;
            }
        }

        // restores old background, so that if another window is drawn, the old one doesn't get shown
        {
            let mut draw = rl.begin_drawing(thread);
            draw.draw_texture(get_expect!(self.background), 0, 0, Color::WHITE);
        }

        rl.set_trace_log(TraceLogLevel::LOG_NONE);
        rl.set_target_fps(self.target_fps);
        rl.set_trace_log(LOG_LEVEL);
        
        if quit {
            Err(ExecutionInterrupt::Quit)
        } else {
            Ok(())
        }
    }

    pub fn draw_once(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        if rl.window_should_close() {
            return Err(ExecutionInterrupt::Quit);
        }

        get_expect_mut!(self.renderer).update(&mut self.imgui, rl);
        (self.build_fn)(self);
        let mut draw = rl.begin_drawing(thread);
        draw.draw_texture(get_expect!(self.background), 0, 0, Color::WHITE);
        get_expect!(self.renderer).render(&mut self.imgui, &mut draw);
        Ok(())
    }
}

#[macro_export]
macro_rules! imgui_centered_text {
    ($ui: ident, $text: expr) => {
        {
            let pos = $ui.cursor_pos();
            $ui.set_cursor_pos([($ui.window_content_region_max()[0] - $ui.calc_text_size($text)[0]) / 2.0, pos[1]]);
            $ui.text($text);
        }
    };
}

#[macro_export]
macro_rules! imgui_spaced_separator {
    ($ui: ident) => {
        $ui.spacing();
        $ui.separator();
        $ui.spacing();
    };
}

#[macro_export]
macro_rules! imgui_items_to_end {
    ($ui: ident) => {
        {
            let available = $ui.window_content_region_max()[1] - $ui.window_content_region_min()[1];
            (available / ($ui.text_line_height() + $crate::gui::Gui::ITEMS_Y_ADJUST)) as i32
        }
    };
}
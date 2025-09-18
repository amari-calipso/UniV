// removes warnings when compiling lite or dev versions
#![cfg_attr(feature = "lite", allow(unused))]
#![cfg_attr(feature = "dev", allow(unused))]

#[cfg(not(any(feature = "full", feature = "lite", feature = "dev")))]
compile_error!("UniV must be compiled with one of these features: \"full\", \"lite\", \"dev\" (\"full\" is default)");

use std::{cell::{OnceCell, RefCell}, cmp::{max, min}, collections::{HashMap, HashSet, VecDeque}, env, fmt::Debug, fs::{self, create_dir, File}, io::{Error, ErrorKind, Read, Write}, panic, path::PathBuf, process::{Child, Command, Stdio}, rc::Rc, sync::{atomic::{self, AtomicBool}, Arc, OnceLock}, thread, time::{Duration, Instant}};
use algos::{Distribution, PivotSelection, Rotation, Shuffle, Sort};
use automation::{AutomationFile, AutomationInterpreter, AutomationMode};
use gui::{AutomationFileInfo, Gui};
use heatmap::HeatMap;
use univm::{bytecode::Bytecode, object::{AnyCallable, AnyObject, Callable, ExecutionInterrupt, List, NativeCallable, UniLValue}, Task, UniVM};
use mixer::SoundMixer;
use nohash_hasher::BuildNoHashHasher;
use num_format::{Locale, ToFormattedString};
use ordered_float::OrderedFloat;
use rand::{rngs::ThreadRng, Rng};

use raylib::{color::Color, ffi::{KeyboardKey, PixelFormat, TraceLogLevel}, math::{Rectangle, Vector2}, prelude::{RaylibDraw, RaylibTextureModeExt}, text::WeakFont, texture::{Image, RenderTexture2D}, RaylibHandle, RaylibThread};
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};
use sounds::AnySound;
use visuals::AnyVisual;
use visual::Visual;
use sound::Sound;
use highlights::HighlightInfo;
use value::{Value, VerifyValue};
use settings::{Profile, UniVSettings};
use unil::ast::Expression;
use utils::report_errors;
use compiler::type_system::UniLType;

#[cfg(feature = "lite")]
use bincode::decode_from_slice;

const LOG_LEVEL: TraceLogLevel = TraceLogLevel::LOG_INFO;

pub const DEFAULT_SOUNDFONT: &[u8] = include_bytes!("../resources/sfx.sf2");
pub const DEFAULT_IMAGE: &[u8] = include_bytes!("../resources/summer_sorting.jpg");
pub const DEFAULT_IMAGE_FORMAT: &str = ".jpg";

pub const DEFAULT_FONT_DATA: &[u8] = include_bytes!("../resources/font.ttf"); // https://github.com/evilmartians/mono
const FONT_FORMAT: &str = ".ttf";
pub const DEFAULT_SPACING_X: f32 = 0.5;
pub const DEFAULT_SPACING_Y: f32 = 2.0;

const LOGO: &[u8] = include_bytes!("../resources/gen/logo.png");
const LOGO_FORMAT: &str = ".png";

#[cfg(not(feature = "lite"))]
const COMPILATION_HEADER: &str = include_str!("header.uni");
#[cfg(feature = "lite")]
const BYTECODE: &[u8] = include_bytes!("../algos.unib");

const RUN_ALL_SORTS_FILENAME: &str = "runAllSorts.ual";
const RUN_ALL_SHUFFLES_FILENAME: &str = "runAllShuffles.ual";

const SAMPLE_RATE: u32 = 48000;
const POLYPHONY_LIMIT: usize = 4;
const STATS_POS: Vector2 = Vector2 { x: 2.0, y: 2.0 };
const STATS_OUTLINE_SIZE: usize = 1;
const LOCALE: Locale = Locale::en;
const UNIT_SAMPLE_DURATION: f64 = 1.0 / 30.0;
const REALTIME_UNIT_SAMPLE_DURATION: f64 = UNIT_SAMPLE_DURATION * 1.2;
const KILLER_INPUT_SPEED: f64 = 500.0;

const SORTED_COLOR:   Color = Color::new(0, 255, 0, 255);
const UNSTABLE_COLOR: Color = Color::YELLOW;
const UNSORTED_COLOR: Color = Color::RED;
const STABLE_COLOR:   Color = Color::BLUE;

// hypotetical maximum framerate the program can run at. not a strict limit,
// it's just used for internal computations to avoid the program getting stuck in the frame dropping logic
const MAX_FRAMERATE: f64 = 4000.0;
// similarly, if we're running at a high framerate, and each frame takes more than this amount to process,
// we're probably not going fast enough and need to run slower
const RENDER_EACH_MAX_SECS: f64 = 1.0;
const MIN_FRAME_TIME: f64 = 1.0 / 10.0;

const REFERENCE_FRAMERATE: u32 = 120;
const REFERENCE_FRAMETIME: f64 = 1.0 / REFERENCE_FRAMERATE as f64;
const SOUND_CHANNELS: u16 = 1;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub static PROGRAM_DIR: OnceLock<PathBuf> = OnceLock::new();

mod utils;
mod highlights;
mod value;
mod visual;
mod visuals;
mod sound;
mod sounds;
mod settings;
mod unil;
mod compiler;
mod univm;
mod algos;
mod mixer;
mod gui;
mod heatmap;
mod api_layers;
mod automation;
mod ffmpeg;

#[cfg(not(feature = "lite"))]
mod language_layers;

#[cfg(feature = "dev")]
mod dev;

enum ArrayState {
    Unsorted,
    Sorted,
    StablySorted,
    ContentsChanged
}

pub type IdentityHashMap<K, V> = HashMap<K, V, BuildNoHashHasher<K>>;
pub type IdentityHashSet<K> = HashSet<K, BuildNoHashHasher<K>>;

pub struct Shared {
    pub array:        Vec<Value>,
    pub aux:          Vec<Value>,
    pub verify_array: Vec<VerifyValue>,

    pub array_max: i64,
    pub aux_max:   i64,

    pub heatmap:     HeatMap,
    pub aux_heatmap: HeatMap,

    pub fps: u32,
    pub reverb: bool,
}

#[derive(Debug)]
struct FFMpeg {
    pub video: Child,
    pub audio: Child
}

#[derive(Debug)]
struct Render {
    pub active: bool,

    pub frame_duration: f64,

    pub recording_duration: Duration,
    pub timestamp_file: OnceCell<File>,

    pub speed_cnt:     u32,
    pub speed_cnt_max: u32,

    pub ffmpeg: OnceCell<FFMpeg>,
    pub ffmpeg_executable: PathBuf
}

struct Audio {
    pub _stream: OutputStream,
    pub handle:  OutputStreamHandle,
}

impl std::fmt::Debug for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Audio").finish()
    }
}

pub type LanguageLayerFn = fn(String, Rc<str>) -> Result<Vec<Expression>, Vec<String>>;

pub struct UniV {
    array: Rc<RefCell<AnyObject>>,

    aux_arrays:    Vec<Rc<RefCell<AnyObject>>>,
    aux_ids:       HashMap<*const AnyObject, usize>,
    non_orig_auxs: HashSet<*const AnyObject>,

    visuals:     Vec<AnyVisual>,
    curr_visual: usize,
    prepared:    bool,

    sounds:     Vec<AnySound>,
    curr_sound: usize,

    distributions: HashMap<Rc<str>, Distribution>,
    shuffles:      HashMap<Rc<str>, Shuffle>,

    sorts:      HashMap<Rc<str>, HashMap<Rc<str>, Sort>>,
    categories: Vec<Rc<str>>,

    pivot_selections: HashMap<Rc<str>, PivotSelection>,
    rotations:        HashMap<Rc<str>, Rotation>,

    run_all_sorts:    OnceCell<AutomationFile>,
    run_all_shuffles: OnceCell<AutomationFile>,

    vm: UniVM,
    automation_interpreter: AutomationInterpreter,
    language_layers: HashMap<Rc<str>, LanguageLayerFn>,

    settings: UniVSettings,
    profile:  Profile,

    writes:      u64,
    swaps:       u64,
    reads:       u64,
    comparisons: u64,
    time:        f64,

    highlights: Vec<HighlightInfo>,
    marks:      IdentityHashMap<usize, HighlightInfo>,
    keep_empty_frames: bool,

    current_category:  String,
    currently_running: String,

    shared:     Shared,
    gui:        Gui,
    autovalues: VecDeque<UniLValue>,

    /// Determines whether the values given by the user to the GUI API should be stored for automation
    store_user_values: bool,
    /// Stores values given by the user to the GUI API for automation purposes if `store_user_values` is set to `true`
    user_values: Vec<UniLValue>,
    shuffle_automation: OnceCell<AutomationFile>,

    /// Random generator source
    rng: ThreadRng,

    rl_handle: OnceCell<RaylibHandle>,
    rl_thread: OnceCell<RaylibThread>,
    /// Texture used as framebuffer for render mode
    render_texture: OnceCell<RenderTexture2D>,
    audio: OnceCell<Audio>,
    mixer: SoundMixer,
    /// Represents the last timestamp in which a frame played a sound in real time mode.
    /// Used to avoid too many sounds playing at once
    sound_timestamp: Instant,
    audio_output_buf: Vec<u8>,

    // self modifying code is a bit overkill, but why not
    render_fn: fn(&mut Self, Vec<HighlightInfo>) -> Result<(), ExecutionInterrupt>,
    sweep_frame_fn: fn(&mut Self, usize, Color) -> Result<(), ExecutionInterrupt>,

    /// Stores highlights resulting from the `prepare_highlights` phase
    set_hl_buf: HashSet<HighlightInfo>,
    /// Stores highlights resulting from the `adapt_indices` phase
    adapt_hl_buf: Vec<HighlightInfo>,
    /// Stores processed highlights that belong to the main array
    main_hl: IdentityHashMap<usize, Color>,
    /// Stores processed highlights that belong to auxiliary arrays
    aux_hl: IdentityHashMap<usize, Color>,

    /// Render mode data
    render: Render,

    /// Current frame number
    frame_n:     u64,
    heatmap_cnt: u16,
    /// Temporary delay set using the `delay` method
    tmp_sleep:   f64,
    past_sleep:  f64,
    target_fps:  u32,

    font:      OnceCell<WeakFont>,
    font_size: usize,
}

impl UniV {
    pub fn new() -> Self {
        let mut language_layers = HashMap::new();
        #[cfg(not(feature = "lite"))]
        language_layers::define(&mut language_layers);

        UniV {
            array: Rc::new(RefCell::new(List::new().into())),

            aux_arrays: Vec::new(),
            aux_ids: HashMap::new(),
            non_orig_auxs: HashSet::new(),

            visuals: Vec::new(),
            curr_visual: 0,
            prepared: false,

            sounds: Vec::new(),
            curr_sound: 0,

            distributions: HashMap::new(),
            shuffles: HashMap::new(),

            sorts: HashMap::new(),
            categories: Vec::new(),

            pivot_selections: HashMap::new(),
            rotations: HashMap::new(),

            run_all_sorts: OnceCell::new(),
            run_all_shuffles: OnceCell::new(),

            vm: UniVM::new(),
            language_layers,
            automation_interpreter: AutomationInterpreter::new(),

            settings: UniVSettings::new(),
            profile: Profile::new(),

            writes: 0,
            swaps: 0,
            reads: 0,
            comparisons: 0,
            time: 0.0,

            highlights: Vec::new(),
            marks: HashMap::default(),
            keep_empty_frames: false,

            current_category: String::new(),
            currently_running: String::new(),

            shared: Shared::new(),
            gui: Gui::new(),
            autovalues: VecDeque::new(),
            store_user_values: false,
            user_values: Vec::new(),
            shuffle_automation: OnceCell::new(),

            rng: rand::rng(),

            rl_handle: OnceCell::new(),
            rl_thread: OnceCell::new(),
            render_texture: OnceCell::new(),
            audio: OnceCell::new(),
            mixer: SoundMixer::new(),
            sound_timestamp: Instant::now(),
            audio_output_buf: Vec::new(),

            render_fn: Self::render_placeholder,
            sweep_frame_fn: Self::sweep_realtime_frame,

            set_hl_buf: HashSet::new(),
            adapt_hl_buf: Vec::new(),
            main_hl: HashMap::default(),
            aux_hl: HashMap::default(),

            render: Render::new(),

            frame_n: 0,
            heatmap_cnt: 0,
            tmp_sleep: 0.0,
            past_sleep: 0.0,
            target_fps: 0,

            font: OnceCell::new(),
            font_size: 12
        }
    }
}

impl Shared {
    pub fn new() -> Self {
        Shared {
            array: Vec::new(),
            aux: Vec::new(),
            verify_array: Vec::new(),

            array_max: 1,
            aux_max: 1,

            heatmap: HeatMap::new(),
            aux_heatmap: HeatMap::new(),

            fps: REFERENCE_FRAMERATE,
            reverb: false
        }
    }
}

impl Render {
    pub fn new() -> Self {
        Render {
            active: false,
            frame_duration: 0.0,
            recording_duration: Duration::ZERO,
            timestamp_file: OnceCell::new(),
            speed_cnt: 0,
            speed_cnt_max: 1,
            ffmpeg: OnceCell::new(),
            ffmpeg_executable: PathBuf::from("ffmpeg")
        }
    }
}

// curr_visual will never be out of bounds unless there's a bug in the program's core.
// so this is safe enough, and it gives us a speed boost in graphics routines, where we need it the most
macro_rules! get_visual {
    ($slf: expr) => {
        {
            debug_assert!($slf.curr_visual < $slf.visuals.len());
            unsafe { $slf.visuals.get_unchecked_mut($slf.curr_visual) }
        }
    };
}

// same goes for sounds
macro_rules! get_sound {
    ($slf: expr) => {
        {
            debug_assert!($slf.curr_sound < $slf.sounds.len());
            unsafe { $slf.sounds.get_unchecked_mut($slf.curr_sound) }
        }
    };
}

macro_rules! render_stats {
    ($slf: ident, $draw: expr, $fps: expr) => {
        if $slf.settings.show_text {
            let full_running = format!("{}: {}", $slf.current_category, $slf.currently_running);

            let running_text;
            if $slf.current_category == "" {
                running_text = &$slf.currently_running;
            } else {
                running_text = &full_running;
            }

            let time;
            let unit;
            if $slf.time > 1000.0 {
                time = $slf.time / 1000.0;
                unit = "s";
            } else {
                time = $slf.time;
                unit = "ms";
            }

            let len = {
                let array = $slf.array.borrow();
                expect_list!(array).items.len()
            };

            utils::gfx::draw_outline_text(
                format!(
                    concat!(
                        "Array length: {}\n",
                        "{}\n",
                        "\n",
                        "Writes: {}\n",
                        "Swaps: {}\n",
                        "\n",
                        "Reads: {}\n",
                        "Comparisons: {}\n",
                        "\n",
                        "Estimated time elapsed: {:.4} {}"
                    ),
                    len.to_formatted_string(&LOCALE),
                    running_text,
                    $slf.writes.to_formatted_string(&LOCALE),
                    $slf.swaps.to_formatted_string(&LOCALE),
                    $slf.reads.to_formatted_string(&LOCALE),
                    $slf.comparisons.to_formatted_string(&LOCALE),
                    time, unit
                ).as_ref(),
                Vector2 { x: STATS_POS.x, y: STATS_POS.y },
                $slf.font_size, get_expect!($slf.font), Color::WHITE, STATS_OUTLINE_SIZE,
                $draw
            );

            if $slf.settings.internal_info {
                let target_fps;
                let dropped_frames;
                let recording_duration;
                if $slf.render.active {
                    target_fps = "None".into();
                    recording_duration = utils::duration_to_hms(&$slf.render.recording_duration);
                    dropped_frames = (
                        1.0 - min(
                            OrderedFloat($slf.settings.render_fps as f64),
                            OrderedFloat(1.0 / $slf.render.frame_duration)
                        ).0 / (REFERENCE_FRAMERATE * $slf.render.speed_cnt_max) as f64) * 100.0;
                } else {
                    target_fps = $slf.target_fps.to_string();
                    dropped_frames = (1.0 - $fps as f64 / $slf.target_fps as f64) * 100.0;
                    recording_duration = "None".into();
                }

                utils::gfx::draw_outline_text_right(
                    format!(
                        concat!(
                            "{} FPS\n",
                            "Target FPS: {}\n",
                            "Dropped frames: {:.2}%\n",
                            "Frame: {}\n",
                            "Recording duration: {}\n",
                            "Current delay: {:.2} ms\n",
                        ),
                        $fps,
                        target_fps,
                        dropped_frames,
                        $slf.frame_n,
                        recording_duration,
                        $slf.tmp_sleep * 1000.0,
                    ).as_ref(),
                    Vector2 { x: $draw.get_screen_width() as f32 - STATS_POS.x, y: STATS_POS.y },
                    $slf.font_size, get_expect!($slf.font), Color::WHITE, STATS_OUTLINE_SIZE,
                    $draw
                );
            }
        }
    };
}

macro_rules! adapt_idx {
    ($slf: ident, $idx: expr, $aux_id: ident) => {
        {
            let mut idx = 'output: {
                let mut offs = 0usize;
                for array in &$slf.aux_arrays {
                    if array.as_ptr() as *const AnyObject == $aux_id {
                        break 'output offs + $idx;
                    }

                    let borrowed = array.borrow();
                    offs += expect_list!(borrowed).items.len();
                }

                panic!("aux_ids was not syncronized with aux_arrays");
            };

            // weird edge case, this happens if $aux_id points to an auxiliary array of length 0,
            // and that auxiliary is the rightmost one in the visualization (at least, that's what i observed,
            // but it's super rare anyway: i have only seen this happening with MSD radix)
            if idx >= $slf.shared.aux.len() {
                idx = max($slf.shared.aux.len(), 1) - 1;
            }

            idx
        }
    };
}

fn samples_to_buf(buf: &mut Vec<u8>, samples: &Vec<i16>) {
    buf.clear();

    if samples.len() > buf.capacity() {
        buf.reserve(samples.len() - buf.capacity());
    }

    for sample in samples {
        buf.extend(sample.to_le_bytes());
    }
}

impl UniV {
    pub fn reset_stats(&mut self) {
        self.writes = 0;
        self.swaps = 0;
        self.reads = 0;
        self.comparisons = 0;
        self.time = 0.0;
    }

    fn add_sort(&mut self, sort: Sort) -> Result<(), Rc<str>> {
        if let Some(category) = self.sorts.get(&sort.category) {
            if category.contains_key(&sort.list_name) {
                return Err(format!("Sort with name '{}' was already added", sort.list_name).into());
            }
        } else {
            self.categories.push(Rc::clone(&sort.category));
            self.sorts.insert(Rc::clone(&sort.category), HashMap::new());
        }

        self.sorts.get_mut(&sort.category).unwrap().insert(Rc::clone(&sort.list_name), sort);
        Ok(())
    }

    fn add_shuffle(&mut self, shuffle: Shuffle) -> Result<(), Rc<str>> {
        if shuffle.name.as_ref() == "Run automation" {
            return Err(Rc::from("'Run automation' is a reserved shuffle name"));
        }

        if self.shuffles.contains_key(&shuffle.name) {
            return Err(format!("Shuffle with name '{}' was already added", shuffle.name).into());
        }

        self.shuffles.insert(Rc::clone(&shuffle.name), shuffle);
        Ok(())
    }

    fn add_distribution(&mut self, distribution: Distribution) -> Result<(), Rc<str>> {
        if self.distributions.contains_key(&distribution.name) {
            return Err(format!("Distribution with name '{}' was already added", distribution.name).into());
        }

        self.distributions.insert(Rc::clone(&distribution.name), distribution);
        Ok(())
    }

    fn add_pivot_selection(&mut self, pivot_selection: PivotSelection) -> Result<(), Rc<str>> {
        if self.pivot_selections.contains_key(&pivot_selection.name) {
            return Err(format!("Pivot selection with name '{}' was already added", pivot_selection.name).into());
        }

        self.pivot_selections.insert(Rc::clone(&pivot_selection.name), pivot_selection);
        Ok(())
    }

    fn add_rotation(&mut self, rotation: Rotation) -> Result<(), Rc<str>> {
        if self.rotations.contains_key(&rotation.name) {
            return Err(format!("Rotation with name '{}' was already added", rotation.name).into());
        }

        self.rotations.insert(Rc::clone(&rotation.name), rotation);
        Ok(())
    }

    pub fn get_pivot_selection(&mut self, name: &str) -> Result<&AnyCallable, ExecutionInterrupt> {
        if !self.pivot_selections.contains_key(name) {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Unknown pivot selection \"{}\"", name
            ).into())));
        }

        Ok(&self.pivot_selections.get(name).unwrap().callable)
    }

    pub fn get_rotation(&mut self, name: &str) -> Result<&Rotation, ExecutionInterrupt> {
        if !self.rotations.contains_key(name) {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Unknown rotation \"{}\"", name
            ).into())));
        }

        Ok(self.rotations.get(name).unwrap())
    }

    pub fn push_autovalue(&mut self, value: UniLValue) {
        self.autovalues.push_back(value);
    }

    pub fn pop_autovalue(&mut self) -> UniLValue {
        self.autovalues.pop_front().unwrap_or(UniLValue::Null)
    }

    pub fn reset_autovalues(&mut self) {
        self.autovalues.clear();
    }

    fn set_currently_running(&mut self, category: &str, running: &str) {
        self.current_category.clear();
        self.currently_running.clear();
        self.current_category.push_str(category);
        self.currently_running.push_str(running);
    }

    pub fn set_current_name(&mut self, running: &str) {
        self.currently_running.clear();
        self.currently_running.push_str(running);
    }

    fn get_max_and_prepare(&mut self) {
        self.shared.array_max = max(1, self.shared.verify_array.last().unwrap().value);
        get_visual!(self).prepare(&mut self.shared, get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread));
        self.prepared = true;
    }

    #[inline]
    fn get_aux_max(&mut self) {
        self.shared.aux_max = max(self.shared.array_max, self.shared.aux.iter().max().unwrap().value);
    }

    fn draw_array_and_stats(&mut self) {
        let rl = get_expect_mut!(self.rl_handle);
        let fps = rl.get_fps();
        let mut draw = rl.begin_drawing(get_expect!(self.rl_thread));
        draw.clear_background(Color::BLACK);
        self.main_hl.clear();
        get_visual!(self).draw(&mut self.shared, &mut draw, &self.main_hl);
        render_stats!(self, &mut draw, fps);
    }

    pub fn run_distribution(&mut self, distribution: &str, length: usize, unique: usize) -> Result<(), ExecutionInterrupt> {
        if length == 0 {
            return Err(self.vm.create_exception(UniLValue::String(
                Rc::from("Main array length cannot be zero")
            )));
        }

        if !self.distributions.contains_key(distribution) {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Unknown distribution \"{}\"", distribution
            ).into())));
        }

        let previous_render = self.render_fn;
        self.render_fn = Self::render_placeholder; // turns off visualization for distribution

        self.array = Rc::new(RefCell::new(List::from(vec![UniLValue::Null; length]).into()));
        self.set_currently_running("Distributions", distribution);

        self.distributions[distribution].callable.clone()
            .call(self, vec![UniLValue::Object(Rc::clone(&self.array))], &mut Task::empty())?;

        self.render_fn = previous_render;

        {
            let mut borrowed = self.array.borrow_mut();
            let array = expect_list_mut!(borrowed);

            if array.items.len() != length {
                return Err(self.vm.create_exception(UniLValue::String(format!(
                    "Distribution generated wrong array size (expecting {} but got {})",
                    length, array.items.len()
                ).into())));
            }

            let t = length as f64 / unique as f64;
            for i in 0 .. length as i64 {
                let item = array.get(i).unwrap();

                if let UniLValue::Int(x) = item {
                    array.set(
                        i,
                        UniLValue::Value {
                            value: (t * ((x as f64 / t) as i64) as f64 + t / 2.0) as i64,
                            idx: i as usize
                        }
                    ).unwrap();
                } else {
                    return Err(self.vm.create_exception(UniLValue::String(format!(
                        "Distribution algorithm should produce integers (got {} at index {})",
                        item.stringify_type(), i
                    ).into())));
                }
            }
        }

        self.get_verify_array().unwrap(); // upper check makes this never fail
        self.get_max_and_prepare();
        self.draw_array_and_stats();
        self.reset_stats();
        self.save_background();

        Ok(())
    }

    pub fn run_shuffle(&mut self, shuffle: &str) -> Result<(), ExecutionInterrupt> {
        if !self.shuffles.contains_key(shuffle) {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Unknown shuffle \"{}\"", shuffle
            ).into())));
        }

        self.set_currently_running("Shuffles", shuffle);

        let len = {
            let array = self.array.borrow();
            expect_list!(array).items.len()
        };

        self.set_speed(len as f64 / 128.0).unwrap(); // speed is 0 only if length is, and length never is

        self.shuffles[shuffle].callable.clone()
            .call(self, vec![UniLValue::Object(Rc::clone(&self.array))], &mut Task::empty())?;

        {
            let mut borrowed = self.array.borrow_mut();
            let array = expect_list_mut!(borrowed);

            for i in 0 .. len {
                if let UniLValue::Value { value, .. } = array.items[i] {
                    array.items[i] = UniLValue::Value { value, idx: i };
                } else {
                    return Err(self.vm.create_exception(UniLValue::String(format!(
                        "Item of type '{}' (at index {}) is not allowed in main array",
                        array.items[i].stringify_type(), i
                    ).into())));
                }
            }
        }

        self.get_verify_array().unwrap(); // upper check makes this never fail
        get_visual!(self).prepare(&mut self.shared, get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread));

        self.marks.clear();
        self.reset_aux();
        self.draw_array_and_stats();
        self.save_background();

        Ok(())
    }

    fn reset_heatmaps(&mut self) {
        self.shared.heatmap.map.clear();
        self.shared.aux_heatmap.map.clear();
    }

    // sweeps could very well be implemented without all this extra code, however, since sweeps are such a specific case,
    // you can greatly optimize them by making specialized functions.
    // sweeps require an highlight for every position in the array, and the highlight handling part has to do a ton of work
    // in that case. in these optimized sweep functions, we just skip all of it
    fn sweep_realtime_frame(&mut self, index: usize, color: Color) -> Result<(), ExecutionInterrupt> {
        self.shared.heatmap.map.insert(index, heatmap::SWEEP_HEAT);
        self.main_hl.insert(index, color);

        if !self.should_render_realtime() {
            return Ok(());
        }

        self.shared.fps = get_expect!(self.rl_handle).get_fps();

        self.adapt_hl_buf.clear();
        self.adapt_hl_buf.push(HighlightInfo::new(index, None, Some(color), false, false));
        self.play_sound();

        {
            let mut draw = get_expect_mut!(self.rl_handle).begin_drawing(get_expect!(self.rl_thread));
            draw.clear_background(Color::BLACK);
            get_visual!(self).draw(&mut self.shared, &mut draw, &self.main_hl);
            render_stats!(self, &mut draw, self.shared.fps);
        }

        if heatmap::should_tick(self.heatmap_cnt, self.shared.fps) {
            self.heatmap_cnt = 0;
            self.shared.heatmap.tick();
        }

        self.heatmap_cnt += 1;
        self.frame_n = self.frame_n.wrapping_add(1);
        Ok(())
    }

    fn should_render_rendered(&mut self) -> bool {
        // random variance is added so that highlights are evenly shown.
        // for instance, if some highlights are shown on odd frames only,
        // and the target fps is double the render fps, those highlights might never be shown
        if self.render.speed_cnt + self.rng.random_range(0..=1) < self.render.speed_cnt_max {
            self.render.speed_cnt += 1;
            self.frame_n = self.frame_n.wrapping_add(1);
            false
        } else {
            self.render.speed_cnt = 0;
            true
        }
    }

    fn sweep_rendered_frame(&mut self, index: usize, color: Color) -> Result<(), ExecutionInterrupt> {
        self.shared.heatmap.map.insert(index, heatmap::SWEEP_HEAT);
        self.main_hl.insert(index, color);

        if !self.should_render_rendered() {
            return Ok(());
        }

        self.adapt_hl_buf.clear();
        self.adapt_hl_buf.push(HighlightInfo::new(index, None, Some(color), false, false));

        let result_frame_duration = 1.0 / self.settings.render_fps as f64;

        let frame_duration = max(
            OrderedFloat(result_frame_duration),
            OrderedFloat(self.render.frame_duration)
        ).0;

        self.mix_sound(max(OrderedFloat(frame_duration), OrderedFloat(UNIT_SAMPLE_DURATION)).0);

        let width;

        {
            let rl = get_expect_mut!(self.rl_handle);
            rl.set_trace_log(TraceLogLevel::LOG_NONE); // disables logs from loading image from texture and unloading

            let fps = rl.get_fps();
            width = rl.get_screen_width() as u32;
            let height = rl.get_screen_height() as f32;

            let mut draw = rl.begin_texture_mode(
                get_expect!(self.rl_thread), get_expect_mut!(self.render_texture)
            );

            draw.clear_background(Color::BLACK);
            get_visual!(self).draw(&mut self.shared, &mut draw, &self.main_hl);
            render_stats!(self, &mut draw, fps);

            drop(draw);
            let mut draw = rl.begin_drawing(get_expect!(self.rl_thread));
            draw_flipped_texture!(draw, get_expect!(self.render_texture), width, height);
        }

        if heatmap::should_tick(self.heatmap_cnt, self.settings.render_fps) {
            self.heatmap_cnt = 0;
            self.shared.heatmap.tick();
        }

        let raw_texture = self.render_texture.take().unwrap().to_raw();
        let frame = unsafe { raylib::ffi::LoadImageFromTexture(raw_texture.texture) };
        self.render_texture.set(unsafe { RenderTexture2D::from_raw(raw_texture) }).unwrap();

        if self.render.ffmpeg.get().is_none() {
            self.init_ffmpeg(&frame)?;
        }

        let (video_stdin, audio_stdin) = {
            let ffmpeg = get_expect_mut!(self.render.ffmpeg);
            (
                ffmpeg.video.stdin.as_mut()
                    .ok_or_else(|| self.vm.create_exception(UniLValue::String(Rc::from("Could not open ffmpeg video STDIN"))))?,
                ffmpeg.audio.stdin.as_mut()
                    .ok_or_else(|| self.vm.create_exception(UniLValue::String(Rc::from("Could not open ffmpeg audio STDIN"))))?
            )
        };

        let raw_frame = unsafe {
            std::slice::from_raw_parts(frame.data as *const u8, (frame.width * frame.height * 4) as usize)
        };

        let samples_per_frame = SoundMixer::duration_to_samples(result_frame_duration);

        let mut i = 0.0;
        while i < frame_duration {
            for line in raw_frame.chunks(width as usize * 4).rev() {
                video_stdin.write_all(line)
                    .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?;
            }

            self.mixer.get(samples_per_frame);
            samples_to_buf(&mut self.audio_output_buf, &self.mixer.output);

            audio_stdin.write_all(&self.audio_output_buf)
                .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?;

            i += result_frame_duration;
        }

        self.mixer.cleanup();
        unsafe { raylib::ffi::UnloadImage(frame) };
        get_expect_mut!(self.rl_handle).set_trace_log(LOG_LEVEL);

        self.render.speed_cnt += 1;
        self.render.recording_duration += Duration::from_secs_f64(frame_duration);
        self.heatmap_cnt += 1;
        self.frame_n = self.frame_n.wrapping_add(1);
        Ok(())
    }

    fn sweep_frame(&mut self, index: usize, color: Color) -> Result<(), ExecutionInterrupt> {
        if self.should_close() {
            return Err(ExecutionInterrupt::Quit);
        }

        (self.sweep_frame_fn)(self, index, color)
    }

    fn sweep(&mut self) -> Result<ArrayState, ExecutionInterrupt> {
        self.adapt_array()?;
        self.set_speed(self.shared.array.len() as f64 / 128.0).unwrap(); // speed is 0 only if length is, and length never is

        self.main_hl.clear();

        // makes the "should render" test pass in realtime mode
        self.set_hl_buf.insert(HighlightInfo::new(0, None, None, true, false));

        let mut color = SORTED_COLOR;
        
        if self.shared.array.len() != self.shared.verify_array.len() {
            color = UNSORTED_COLOR;
        }

        for i in 0 .. self.shared.array.len() {
            if self.shared.array[i].value != self.shared.verify_array[i].value {
                color = UNSORTED_COLOR;
            }

            self.sweep_frame(i, color)?;
        }

        if color == UNSORTED_COLOR {
            let mut sorted = true;
            for i in 0 .. self.shared.array.len() - 1 {
                if self.shared.array[i].value > self.shared.array[i + 1].value {
                    sorted = false;
                    break;
                }
            }

            return {
                if sorted {
                    Ok(ArrayState::ContentsChanged)
                } else {
                    Ok(ArrayState::Unsorted)
                }
            };
        }

        let mut stability_check: IdentityHashMap<i64, Vec<usize>> = HashMap::default();
        for i in 0 .. self.shared.array.len() {
            let k = self.shared.array[i].value;
            let v = self.shared.array[i].idx;

            if let Some(group) = stability_check.get_mut(&k) {
                group.push(v);
            } else {
                stability_check.insert(k, vec![v]);
            }
        }

        color = STABLE_COLOR;

        let mut ordered_items: Vec<i64> = stability_check.keys().cloned().collect();
        ordered_items.sort_unstable();

        let mut j = 0usize;
        for item in &ordered_items {
            let uniques = stability_check.get(item).unwrap();

            for i in 0 .. uniques.len() - 1 {
                if uniques[i] > uniques[i + 1] {
                    color = UNSTABLE_COLOR;
                }

                self.sweep_frame(j, color)?;
                j += 1;
            }

            self.sweep_frame(j, color)?;
            j += 1;
        }

        self.marks.clear();

        if color == STABLE_COLOR {
            Ok(ArrayState::StablySorted)
        } else {
            Ok(ArrayState::Sorted)
        }
    }

    fn wait(&mut self, mut secs: f64) -> Result<(), ExecutionInterrupt> {
        let old_speed_cnt_max = self.render.speed_cnt_max;
        let old_frame_duration = self.render.frame_duration;
        let old_tmp_sleep = self.tmp_sleep;
        let old_target_fps = self.target_fps;
        let old_keep_empty_frames = self.keep_empty_frames;
        let old_highlights = self.highlights.clone();
        let old_marks = self.marks.clone();

        self.render.speed_cnt_max = 1;
        self.render.speed_cnt = 1;
        self.render.frame_duration = 1.0 / self.settings.render_fps as f64;
        self.tmp_sleep = 0.0;
        self.target_fps = REFERENCE_FRAMERATE;
        self.keep_empty_frames = true;
        self.highlights.clear();
        self.marks.clear();

        if self.render.active {
            while secs > 0.0 {
                self.immediate_highlight(Vec::new())?;
                secs -= self.render.frame_duration;
            }
        } else {
            self.set_target_fps(REFERENCE_FRAMERATE);

            while secs > 0.0 {
                let start_time = Instant::now();
                self.immediate_highlight(Vec::new())?;
                secs -= start_time.elapsed().as_secs_f64();
            }

            self.set_target_fps(old_target_fps);
        }

        self.render.speed_cnt_max = old_speed_cnt_max;
        self.render.frame_duration = old_frame_duration;
        self.tmp_sleep = old_tmp_sleep;
        self.target_fps = old_target_fps;
        self.keep_empty_frames = old_keep_empty_frames;
        self.highlights = old_highlights;
        self.marks = old_marks;

        Ok(())
    }

    fn report_array_state(&mut self) -> Result<(), ExecutionInterrupt> {
        let sort_name = self.currently_running.clone();

        self.set_currently_running("", "Checking...");

        match self.sweep()? {
            ArrayState::Unsorted => {
                self.set_current_name("The list was not sorted");
                log!(TraceLogLevel::LOG_INFO, "{sort_name} has failed");
            }
            ArrayState::ContentsChanged => {
                self.set_current_name("The list's original contents were changed");
                log!(TraceLogLevel::LOG_INFO, "{sort_name} has failed (contents changed)");
            }
            ArrayState::Sorted => {
                self.set_current_name("The list was sorted");
                log!(TraceLogLevel::LOG_INFO, "{sort_name} sorted the list unstably");
            }
            ArrayState::StablySorted => {
                self.set_current_name("The list was sorted stably");
                log!(TraceLogLevel::LOG_INFO, "{sort_name} sorted the list stably");
            }
        }

        self.draw_array_and_stats();
        self.wait(1.25)?;
        self.mixer.reset();

        Ok(())
    }

    pub fn run_sort(&mut self, category: &str, sort: &str) -> Result<(), ExecutionInterrupt> {
        if !self.sorts.contains_key(category) {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Unknown sort category \"{}\"", category
            ).into())));
        }

        if !self.sorts[category].contains_key(sort) {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Unknown sort \"{}\"", sort
            ).into())));
        }

        self.wait(1.25)?;

        self.reset_stats();
        self.set_currently_running(category, &Rc::clone(&self.sorts[category][sort].name));

        self.sorts[category][sort].callable.clone()
            .call(self, vec![UniLValue::Object(Rc::clone(&self.array))], &mut Task::empty())?;

        self.marks.clear();
        self.reset_aux();
        self.report_array_state()?;
        self.reset_heatmaps();
        self.save_background();

        Ok(())
    }

    fn get_verify_array(&mut self) -> Result<(), ExecutionInterrupt> {
        self.adapt_array()?;
        self.shared.verify_array.clear();

        let len = {
            let array = self.array.borrow();
            expect_list!(array).items.len()
        };

        if len > self.shared.verify_array.capacity() {
            self.shared.verify_array.reserve(len - self.shared.verify_array.capacity());
        }

        for item in &self.shared.array {
            self.shared.verify_array.push(VerifyValue::new(item.value, item.idx));
        }

        self.shared.verify_array.sort();
        Ok(())
    }

    fn adapt_array(&mut self) -> Result<(), ExecutionInterrupt> {
        self.shared.array.clear();

        let len = {
            let array = self.array.borrow();
            expect_list!(array).items.len()
        };

        if len > self.shared.verify_array.capacity() {
            self.shared.verify_array.reserve(len - self.shared.verify_array.capacity());
        }

        let array = self.array.borrow();
        for (i, item) in expect_list!(array).items.iter().enumerate() {
            if let UniLValue::Value { value, idx } = item {
                self.shared.array.push(Value::new(*value, *idx, None));
            } else {
                return Err(self.vm.create_exception(UniLValue::String(format!(
                    "Item of type '{}' (at index {}) is not allowed in main array",
                    item.stringify_type(), i
                ).into())));
            }
        }

        Ok(())
    }

    fn prepare_highlights(&mut self, highlights: Vec<HighlightInfo>) -> Result<(), ExecutionInterrupt> {
        self.set_hl_buf.clear();

        for highlight in highlights {
            if let Some(current_highlight) = self.set_hl_buf.get(&highlight) {
                if highlight.is_write && !current_highlight.is_write {
                    self.set_hl_buf.replace(highlight);
                }
            } else {
                self.set_hl_buf.insert(highlight);
            }
        }

        for highlight in &self.highlights {
            if let Some(current_highlight) = self.set_hl_buf.get(&highlight) {
                if highlight.is_write && !current_highlight.is_write {
                    self.set_hl_buf.replace(highlight.clone());
                }
            } else {
                self.set_hl_buf.insert(highlight.clone());
            }
        }

        // removes highlights to auxiliary arrays that are not being visualized
        self.set_hl_buf.retain(|highlight| {
            highlight.aux.is_none_or(|aux| self.aux_ids.contains_key(&aux))
        });

        for highlight in self.set_hl_buf.iter() {
            if highlight.is_write {
                if let Some(aux) = highlight.aux {
                    let adapted_idx = adapt_idx!(self, highlight.idx, aux);
                    self.shared.aux_heatmap.access(adapted_idx);
                } else {
                    self.shared.heatmap.access(highlight.idx);
                }
            }
        }

        Ok(())
    }

    fn prepare_marks(&mut self) {
        for highlight in self.marks.values() {
            if !self.set_hl_buf.contains(&highlight) {
                self.set_hl_buf.insert(highlight.clone());
            }
        }
    }

    fn adapt_indices(&mut self) -> Result<(), ExecutionInterrupt> {
        self.adapt_hl_buf.clear();

        for highlight in self.set_hl_buf.iter() {
            let mut tmp = highlight.clone();
            let aux = tmp.aux.is_some();

            if aux && self.settings.show_aux {
                let aux_id = tmp.aux.unwrap();
                tmp.idx = adapt_idx!(self, tmp.idx, aux_id);

                if tmp.idx >= self.shared.aux.len() {
                    return Err(self.vm.create_exception(UniLValue::String(format!(
                        "Highlight index {} is out of bounds for adapted auxiliary of length {}",
                        tmp.idx, self.shared.aux.len()
                    ).into())));
                }
            } else {
                let array_len = self.shared.array.len();

                if aux {
                    tmp.idx %= array_len;
                }

                if tmp.idx >= array_len {
                    return Err(self.vm.create_exception(UniLValue::String(format!(
                        "Highlight index {} is out of bounds for array of length {}",
                        tmp.idx, self.shared.aux.len()
                    ).into())));
                }
            }

            self.adapt_hl_buf.push(tmp);
        }

        Ok(())
    }

    fn partition_indices(&mut self) -> Result<(), ExecutionInterrupt> {
        self.main_hl.clear();
        self.aux_hl.clear();

        for highlight in &self.adapt_hl_buf {
            if highlight.aux.is_some() {
                if let Some(color) = highlight.color {
                    self.aux_hl.insert(highlight.idx, color);
                } else {
                    self.aux_hl.insert(highlight.idx, get_visual!(self).highlight_color());
                }
            } else {
                if let Some(color) = highlight.color {
                    self.main_hl.insert(highlight.idx, color);
                } else {
                    self.main_hl.insert(highlight.idx, get_visual!(self).highlight_color());
                }
            }
        }

        Ok(())
    }

    fn refresh_aux_ids(&mut self) {
        for (i, aux) in self.aux_arrays.iter().enumerate() {
            self.aux_ids.insert(aux.as_ptr() as *const AnyObject, i);
        }
    }

    fn garbage_collect(&mut self) {
        let old_aux_ids_len = self.aux_ids.len();
        for array in &self.aux_arrays {
            if Rc::strong_count(&array) == 1 {
                self.aux_ids.remove(&(array.as_ptr() as *const AnyObject));
            }
        }

        if self.aux_ids.len() != old_aux_ids_len {
            self.aux_arrays.retain(|aux| Rc::strong_count(&aux) != 1);

            if self.aux_arrays.is_empty() {
                self.on_aux_off();
            }
        }

        self.refresh_aux_ids();
    }

    fn prepare_aux(&mut self) -> Result<bool, ExecutionInterrupt> {
        let mut aux = self.settings.show_aux && self.aux_arrays.len() != 0;

        if aux {
            self.garbage_collect();
            aux &= self.aux_arrays.len() != 0;
        }

        if aux {
            let old_len = self.shared.aux.len();
            self.shared.aux.clear();

            for array in &self.aux_arrays {
                if let AnyObject::List(list) = &*array.borrow() {
                    if self.non_orig_auxs.contains(&(array.as_ptr() as *const AnyObject)) {
                        let mut max;
                        if let UniLValue::Value { value, .. } = &list.items[0] {
                            max = *value;
                        } else {
                            return Err(self.vm.create_exception(UniLValue::String(format!(
                                "Item of type '{}' (at index 0) is not allowed in visualized auxiliary arrays",
                                list.items[0].stringify_type()
                            ).into())));
                        }

                        for i in 1 .. list.items.len() {
                            if let UniLValue::Value { value, .. } = &list.items[i] {
                                if *value > max {
                                    max = *value;
                                }
                            } else {
                                return Err(self.vm.create_exception(UniLValue::String(format!(
                                    "Item of type '{}' (at index {}) is not allowed in visualized auxiliary arrays",
                                    list.items[i].stringify_type(), i
                                ).into())));
                            }
                        }

                        let mlt = {
                            if max == 0 || self.shared.aux_max == 0 {
                                1.0
                            } else {
                                self.shared.aux_max as f64 / (max as f64 * 1.1)
                            }
                        };

                        for orig in &list.items {
                            if let UniLValue::Value { value, idx } = orig {
                                self.shared.aux.push(Value {
                                    value: {
                                        if *value <= 0 {
                                            0
                                        } else {
                                            (*value as f64 * mlt) as i64
                                        }
                                    },
                                    idx: *idx,
                                    aux: Some(array.as_ptr() as *const AnyObject)
                                });
                            } else {
                                unreachable!()
                            }
                        }
                    } else {
                        for orig in &list.items {
                            if let UniLValue::Value { value, idx } = orig {
                                self.shared.aux.push(Value {
                                    value: {
                                        if *value <= 0 {
                                            0
                                        } else {
                                            *value
                                        }
                                    },
                                    idx: *idx,
                                    aux: Some(array.as_ptr() as *const AnyObject)
                                });
                            } else {
                                unreachable!()
                            }
                        }
                    }
                }
            }

            if self.shared.aux.len() == 0 {
                self.shared.aux.push(Value {
                    value: 0, idx: 0,
                    aux: Some(self.aux_arrays.first().unwrap().as_ptr() as *const AnyObject)
                });
            }

            let old_max = self.shared.aux_max;
            self.get_aux_max();

            if old_max != self.shared.aux_max || old_len != self.shared.aux.len() {
                get_visual!(self).on_aux_on(&mut self.shared, get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread));
            }
        }

        Ok(aux)
    }

    // disables annoying timer log
    fn set_target_fps(&mut self, fps: u32) {
        self.gui.target_fps = fps;
        let rl = get_expect_mut!(self.rl_handle);
        rl.set_trace_log(TraceLogLevel::LOG_NONE);
        rl.set_target_fps(fps);
        rl.set_trace_log(LOG_LEVEL);
    }

    pub fn set_speed(&mut self, speed: f64) -> Result<(), Rc<str>> {
        if speed == 0.0 {
            return Err(Rc::from("Speed setting cannot be zero"));
        }

        let ref_fps_f64 = REFERENCE_FRAMERATE as f64;

        if self.render.active {
            let render_fps_f64 = self.settings.render_fps as f64;
            let render_speed = speed * (ref_fps_f64 / render_fps_f64);

            if render_speed >= 1.0 {
                self.render.speed_cnt_max = render_speed.round() as u32;
                self.render.frame_duration = 1.0 / render_fps_f64;
            } else {
                self.render.speed_cnt_max = 0;
                self.render.frame_duration = 0.001 / render_speed;
            }

            self.render.speed_cnt = 1;

            // no framerate limit for rendering, just go as fast as you can
            self.target_fps = u32::MAX;
        } else {
            self.target_fps = (speed * ref_fps_f64) as u32;
        }

        self.set_target_fps(self.target_fps);

        Ok(())
    }

    // TODO: check if this is correct
    pub fn get_speed(&self) -> f64 {
        let ref_fps_f64 = REFERENCE_FRAMERATE as f64;

        if self.render.active {
            let ref_over_render = ref_fps_f64 / self.settings.render_fps as f64;

            if self.render.speed_cnt_max == 0 {
                (self.render.frame_duration * 1000.0) / ref_over_render
            } else {
                self.render.speed_cnt_max as f64 / ref_over_render
            }
        } else {
            self.target_fps as f64 / ref_fps_f64
        }
    }

    // TODO: check if this is correct
    pub fn delay(&mut self, amt_ms: f64) {
        self.tmp_sleep = max(OrderedFloat(0.0), OrderedFloat(amt_ms / (1000.0 * self.get_speed()))).0;
    }

    pub fn reset_speed(&mut self) {
        self.target_fps = REFERENCE_FRAMERATE;
        self.render.speed_cnt = 1;
        self.render.frame_duration = 1.0 / self.settings.render_fps as f64;
    }

    #[inline]
    fn on_aux_off(&mut self) {
        self.shared.aux.clear();
        get_visual!(self).on_aux_off(&mut self.shared, get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread));
    }

    pub fn reset_aux(&mut self) {
        self.aux_arrays.clear();
        self.aux_ids.clear();
        self.non_orig_auxs.clear();
        self.on_aux_off();
    }

    pub fn remove_aux(&mut self, aux: &Rc<RefCell<AnyObject>>) {
        let old_len = self.aux_arrays.len();

        if let Some(idx) = self.aux_ids.remove(&(aux.as_ptr() as *const AnyObject)) {
            self.aux_arrays.remove(idx);

            if idx != self.aux_arrays.len() {
                self.refresh_aux_ids();
            }
        }

        if self.aux_arrays.len() == 0 && old_len != 0 {
            self.on_aux_off();
        }
    }

    pub fn get_optional_aux_id(&self, aux_id: *const AnyObject) -> Option<*const AnyObject> {
        if aux_id == self.array.as_ptr() as *const AnyObject {
            None
        } else {
            Some(aux_id)
        }
    }

    pub fn add_aux(&mut self, aux: &Rc<RefCell<AnyObject>>) -> Result<(), ExecutionInterrupt> {
        let aux_id = aux.as_ptr() as *const AnyObject;

        if self.get_optional_aux_id(aux_id).is_none() {
            return Err(self.vm.create_exception(UniLValue::String(
                Rc::from("Cannot add main array to auxiliaries")
            )));
        }

        if self.aux_ids.contains_key(&aux_id) {
            return Err(self.vm.create_exception(UniLValue::String(
                Rc::from("Cannot add same auxiliary array to visualization multiple times")
            )));
        }

        self.aux_ids.insert(aux_id, self.aux_arrays.len());
        self.aux_arrays.push(Rc::clone(&aux));

        Ok(())
    }

    pub fn standalone_array(&mut self, length: usize) -> Rc<RefCell<AnyObject>> {
        let mut list = List::from(Vec::with_capacity(length));

        for i in 0 .. length {
            list.items.push(UniLValue::Value { value: 0, idx: i });
        }

        Rc::new(RefCell::new(list.into()))
    }

    pub fn create_aux(&mut self, length: usize) -> UniLValue {
        let output = self.standalone_array(length);
        self.add_aux(&output).unwrap(); // since the array has been created right now, this never fails
        UniLValue::Object(output)
    }

    pub fn set_non_orig_aux(&mut self, aux: &Rc<RefCell<AnyObject>>) {
        self.non_orig_auxs.insert(aux.as_ptr() as *const AnyObject);
    }

    fn render_placeholder(&mut self, _: Vec<HighlightInfo>) -> Result<(), ExecutionInterrupt> {
        self.highlights.clear();
        Ok(())
    }

    fn get_value_and_max_from_highlight(&self, highlight: &HighlightInfo) -> (i64, i64) {
        if self.settings.show_aux && highlight.aux.is_some() && self.aux_arrays.len() != 0 {
            (self.shared.aux[highlight.idx].value, self.shared.aux_max)
        } else {
            (self.shared.array[highlight.idx].value, self.shared.array_max)
        }
    }

    fn mix_sound(&mut self, duration: f64) -> usize {
        let mut values = HashSet::new();

        let sample_length = SoundMixer::duration_to_samples(duration);

        let mut amt = 0usize;
        let mut max = 0usize;
        for highlight in &self.adapt_hl_buf {
            if highlight.silent {
                continue;
            }

            let (value, max_value) = self.get_value_and_max_from_highlight(highlight);
            if values.contains(&value) {
                continue;
            }

            values.insert(value);
            let raw = get_sound!(self).play(value, max_value, sample_length);

            if raw.len() > max {
                max = raw.len();
            }

            self.mixer.put(raw);

            amt += 1;

            if amt == POLYPHONY_LIMIT {
                break;
            }
        }

        max
    }

    fn play_sound(&mut self) {
        if self.audio.get().is_some() {
            let sound_duration = max(
                OrderedFloat(REALTIME_UNIT_SAMPLE_DURATION),
                max(
                    OrderedFloat(1.0 / self.target_fps as f64),
                    OrderedFloat(self.tmp_sleep)
                )
            ).0;

            // if the sound is a long one (suggesting a longer frame due to a temporary sleep),
            // play it ignoring the frametime limit
            if sound_duration <= REALTIME_UNIT_SAMPLE_DURATION &&
                self.sound_timestamp.elapsed().as_secs_f64() < REFERENCE_FRAMETIME
            {
                return;
            }

            let samples_to_take = self.mix_sound(sound_duration);
            self.mixer.get(samples_to_take);
            self.mixer.cleanup();

            let sink = Sink::try_new(&get_expect!(self.audio).handle).unwrap();
            sink.append(SamplesBuffer::new(SOUND_CHANNELS, SAMPLE_RATE, self.mixer.output.clone()));
            sink.detach();

            self.sound_timestamp = Instant::now();
        }
    }

    fn clear_highlights_if_precise(&mut self) {
        if self.settings.precise_highlights {
            self.highlights.clear();
        }
    }

    fn should_render_realtime(&mut self) -> bool {
        if !self.keep_empty_frames && self.set_hl_buf.is_empty() {
            return false;
        }

        let frame_time = min(
            max(
                OrderedFloat(0.0),
                OrderedFloat(get_expect!(self.rl_handle).get_frame_time() as f64 - self.past_sleep)
            ),
            // if it gets this slow, don't adjust for fps, otherwise the program will get stuck
            OrderedFloat(MIN_FRAME_TIME)
        ).0;
        self.past_sleep = 0.0;

        let mut render_each = max(1, (frame_time * self.target_fps as f64).round() as u64);

        // if the amount of frames to skip gets too high, the program will get stuck
        // so find a lower amount of frames to skip, in a "best effort" approach
        if render_each != 1 && render_each as f64 / MAX_FRAMERATE >= RENDER_EACH_MAX_SECS {
            let mut a = 1;
            let mut b = render_each;

            while a < b {
                let m = a + (b - a) / 2;

                if m as f64 / MAX_FRAMERATE >= RENDER_EACH_MAX_SECS {
                    b = m;
                } else {
                    a = m + 1;
                }
            }

            render_each = a;
        }

        if self.frame_n % render_each == 0 {
            true
        } else {
            self.clear_highlights_if_precise();
            self.frame_n = self.frame_n.wrapping_add(1);
            false
        }
    }

    fn handle_stop(&self) -> Result<(), ExecutionInterrupt> {
        if get_expect!(self.rl_handle).is_key_pressed(KeyboardKey::KEY_S) {
            Err(ExecutionInterrupt::StopAlgorithm)
        } else {
            Ok(())
        }
    }

    fn realtime_highlight(&mut self, highlights: Vec<HighlightInfo>) -> Result<(), ExecutionInterrupt> {
        self.handle_stop()?;
        self.prepare_highlights(highlights)?;

        if !self.should_render_realtime() {
            return Ok(());
        }

        self.prepare_marks();
        self.adapt_array()?;
        let aux = self.prepare_aux()?;
        self.adapt_indices()?;
        self.play_sound();
        self.partition_indices()?;

        self.shared.fps = get_expect!(self.rl_handle).get_fps();
        let visual = get_visual!(self);

        visual.pre_draw(&mut self.shared, get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread));

        {
            let mut draw = get_expect_mut!(self.rl_handle).begin_drawing(get_expect!(self.rl_thread));
            draw.clear_background(Color::BLACK);

            visual.draw(&mut self.shared, &mut draw, &self.main_hl);

            if aux {
                visual.draw_aux(&mut self.shared, &mut draw, &self.aux_hl);
            }

            render_stats!(self, &mut draw, self.shared.fps);
        }

        if heatmap::should_tick(self.heatmap_cnt, self.shared.fps) {
            self.heatmap_cnt = 0;
            self.shared.heatmap.tick();

            if aux {
                self.shared.aux_heatmap.tick();
            }
        }

        self.heatmap_cnt += 1;
        self.frame_n = self.frame_n.wrapping_add(1);
        self.highlights.clear();

        let sleep = self.tmp_sleep - get_expect!(self.rl_handle).get_frame_time() as f64;
        if sleep > 0.0 {
            thread::sleep(Duration::from_secs_f64(sleep));
            self.past_sleep = sleep;
        }

        self.tmp_sleep = 0.0;
        Ok(())
    }

    fn init_ffmpeg(&mut self, frame: &raylib::ffi::Image) -> Result<(), ExecutionInterrupt> {
        self.render.ffmpeg.set(FFMpeg {
            video: {
                Command::new(&self.render.ffmpeg_executable)
                    .args([
                        "-hwaccel", "auto",

                        // video input
                        "-f", "rawvideo",
                        "-pix_fmt", "rgba",
                        "-s", &format!("{}x{}", frame.width, frame.height),
                        "-r", &self.settings.render_fps.to_string(),
                        "-i", "-",

                        // output
                        "-b:v", &format!("{}k", self.settings.bitrate),
                        "-c:v", &self.profile.codec,
                        "-profile:v", &self.profile.profile,
                        "-pix_fmt", &self.profile.pix_fmt,
                        "-preset", &self.profile.preset,
                        "-tune", &self.profile.tune,
                        "-y", "tmp.mp4",
                    ])
                    .stdin(Stdio::piped())
                    .spawn()
                    .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?
            },
            audio: {
                Command::new(&self.render.ffmpeg_executable)
                    .args([
                        "-hwaccel", "auto",

                        // audio input
                        "-f", "s16le",
                        "-ar", &SAMPLE_RATE.to_string(),
                        "-ac", &SOUND_CHANNELS.to_string(),
                        "-i", "-",

                        // output
                        "-c:a", "pcm_s16le",
                        "-y", "tmp.wav",
                    ])
                    .stdin(Stdio::piped())
                    .spawn()
                    .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?
            }
        }).expect("ffmpeg OnceCell was already set");

        Ok(())
    }

    fn rendered_highlight(&mut self, highlights: Vec<HighlightInfo>) -> Result<(), ExecutionInterrupt> {
        self.handle_stop()?;
        self.prepare_highlights(highlights)?;

        if !self.keep_empty_frames && self.set_hl_buf.is_empty() {
            self.frame_n = self.frame_n.wrapping_add(1);
            return Ok(());
        }

        if !self.should_render_rendered() {
            self.clear_highlights_if_precise();
            return Ok(());
        }

        self.prepare_marks();
        self.adapt_array()?;
        let aux = self.prepare_aux()?;
        self.adapt_indices()?;

        let result_frame_duration = 1.0 / self.settings.render_fps as f64;

        let frame_duration = max(
            OrderedFloat(result_frame_duration),
            OrderedFloat(self.render.frame_duration + self.tmp_sleep)
        ).0;

        self.mix_sound(max(OrderedFloat(frame_duration), OrderedFloat(UNIT_SAMPLE_DURATION)).0);
        self.partition_indices()?;

        let width;

        let visual = get_visual!(self);
        visual.pre_draw(&mut self.shared, get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread));

        {
            let rl = get_expect_mut!(self.rl_handle);
            rl.set_trace_log(TraceLogLevel::LOG_NONE); // disables logs from loading image from texture and unloading

            let fps = rl.get_fps();

            width = rl.get_screen_width() as u32;
            let height = rl.get_screen_height() as f32;

            let mut draw = rl.begin_texture_mode(
                get_expect!(self.rl_thread), get_expect_mut!(self.render_texture)
            );

            draw.clear_background(Color::BLACK);

            visual.draw(&mut self.shared, &mut draw, &self.main_hl);

            if aux {
                visual.draw_aux(&mut self.shared, &mut draw, &self.aux_hl);
            }

            render_stats!(self, &mut draw, fps);

            drop(draw);
            let mut draw = rl.begin_drawing(get_expect!(self.rl_thread));
            draw_flipped_texture!(draw, get_expect!(self.render_texture), width, height);
        }

        if heatmap::should_tick(self.heatmap_cnt, self.settings.render_fps) {
            self.heatmap_cnt = 0;
            self.shared.heatmap.tick();

            if aux {
                self.shared.aux_heatmap.tick();
            }
        }

        let raw_texture = self.render_texture.take().unwrap().to_raw();
        let frame = unsafe { raylib::ffi::LoadImageFromTexture(raw_texture.texture) };
        self.render_texture.set(unsafe { RenderTexture2D::from_raw(raw_texture) }).unwrap();

        if self.render.ffmpeg.get().is_none() {
            self.init_ffmpeg(&frame)?;
        }

        let (video_stdin, audio_stdin) = {
            let ffmpeg = get_expect_mut!(self.render.ffmpeg);
            (
                ffmpeg.video.stdin.as_mut()
                    .ok_or_else(|| self.vm.create_exception(UniLValue::String(Rc::from("Could not open ffmpeg video STDIN"))))?,
                ffmpeg.audio.stdin.as_mut()
                    .ok_or_else(|| self.vm.create_exception(UniLValue::String(Rc::from("Could not open ffmpeg audio STDIN"))))?
            )
        };

        let raw_frame = unsafe {
            std::slice::from_raw_parts(frame.data as *const u8, (frame.width * frame.height * 4) as usize)
        };

        let samples_per_frame = SoundMixer::duration_to_samples(result_frame_duration);

        let mut i = 0.0;
        while i < frame_duration {
            for line in raw_frame.chunks(width as usize * 4).rev() {
                video_stdin.write_all(line)
                    .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?;
            }

            self.mixer.get(samples_per_frame);
            samples_to_buf(&mut self.audio_output_buf, &self.mixer.output);

            audio_stdin.write_all(&self.audio_output_buf)
                .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?;

            i += result_frame_duration;
        }

        self.mixer.cleanup();
        unsafe { raylib::ffi::UnloadImage(frame) };
        get_expect_mut!(self.rl_handle).set_trace_log(LOG_LEVEL);

        self.render.speed_cnt += 1;
        self.render.recording_duration += Duration::from_secs_f64(frame_duration);
        self.heatmap_cnt += 1;
        self.frame_n = self.frame_n.wrapping_add(1);
        self.highlights.clear();
        self.tmp_sleep = 0.0;
        Ok(())
    }

    pub fn immediate_highlight(&mut self, highlights: Vec<HighlightInfo>) -> Result<(), ExecutionInterrupt> {
        if self.should_close() {
            return Err(ExecutionInterrupt::Quit);
        }

        (self.render_fn)(self, highlights)
    }

    pub fn set_visual_by_id(&mut self, id: usize) -> Result<(), ExecutionInterrupt> {
        if id >= self.visuals.len() {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Invalid visual ID '{}' (length is {})", id, self.visuals.len()
            ).into())));
        }

        log!(TraceLogLevel::LOG_INFO, "Visual has been set to \"{}\" ID: {}", self.visuals[id].name(), id);
        self.curr_visual = id;
        self.prepared = false;
        Ok(())
    }

    pub fn set_visual_by_name(&mut self, name: &str) -> Result<(), ExecutionInterrupt> {
        if let Ok(id) = self.visuals.binary_search_by(|x| x.name().cmp(name)) {
            log!(TraceLogLevel::LOG_INFO, "Visual has been set to \"{}\" ID: {}", self.visuals[id].name(), id);
            self.curr_visual = id;
            self.prepared = false;
            return Ok(());
        }

        Err(self.vm.create_exception(UniLValue::String(format!("Unknown visual \"{}\"", name).into())))
    }

    pub fn set_sound_by_id(&mut self, id: usize) -> Result<(), ExecutionInterrupt> {
        if id >= self.sounds.len() {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Invalid sound ID '{}' (length is {})", id, self.sounds.len()
            ).into())));
        }

        log!(TraceLogLevel::LOG_INFO, "Sound has been set to \"{}\" ID: {}", self.sounds[id].name(), id);
        self.curr_sound = id;
        get_sound!(self).prepare(&self.shared, &mut self.gui, get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread))?;
        Ok(())
    }

    pub fn set_sound_by_name(&mut self, name: &str) -> Result<(), ExecutionInterrupt> {
        if let Ok(id) = self.sounds.binary_search_by(|x| x.name().cmp(name)) {
            log!(TraceLogLevel::LOG_INFO, "Sound has been set to \"{}\" ID: {}", self.sounds[id].name(), id);
            self.curr_sound = id;
            get_sound!(self).prepare(&self.shared, &mut self.gui, get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread))?;
            return Ok(());
        }

        Err(self.vm.create_exception(UniLValue::String(format!("Unknown sound \"{}\"", name).into())))
    }

    fn try_save_settings(&mut self) {
        log!(TraceLogLevel::LOG_INFO, "Saving settings");

        if let Err(e) = self.settings.save() {
            log!(TraceLogLevel::LOG_ERROR, "Could not save settings");
            log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
        }
    }

    fn sync_reverb(&mut self) {
        self.shared.reverb = self.settings.reverb;
    }

    fn try_load_current_profile(&mut self) {
        log!(TraceLogLevel::LOG_INFO, "Loading render profile");

        match Profile::load(&self.settings.profile) {
            Ok(profile) => self.profile = profile,
            Err(e) => {
                log!(TraceLogLevel::LOG_ERROR, "Could not load chosen profile");
                log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
            }
        }
    }

    fn load_settings(&mut self, first: bool) {
        log!(TraceLogLevel::LOG_INFO, "Loading settings");
        match UniVSettings::load() {
            Ok(settings) => self.settings = settings,
            Err(e) => {
                if first {
                    let settings_folder = program_dir!().join("config");
                    let no_settings_folder = !settings_folder.exists();

                    if no_settings_folder || matches!(e.kind(), ErrorKind::NotFound) {
                        log!(TraceLogLevel::LOG_WARNING, "Could not find user settings file. Assuming first start");

                        if no_settings_folder {
                            log!(TraceLogLevel::LOG_INFO, "Creating config folder");

                            if let Err(e) = create_dir(&settings_folder) {
                                log!(TraceLogLevel::LOG_ERROR, "Could not create config folder");
                                log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                                return;
                            }
                        }

                        self.try_save_settings();
                    }
                } else {
                    log!(TraceLogLevel::LOG_ERROR, "Could not load user settings");
                    log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                    log!(TraceLogLevel::LOG_WARNING, "Using default settings");
                }
            }
        }

        self.sync_reverb();
        self.try_load_current_profile();
    }

    fn load_algo_folder(&mut self, folder: &str) -> (Vec<Expression>, Vec<Error>) {
        let mut errors = Vec::new();
        let mut dir_ast = Vec::new();

        let full_path = program_dir!().join("algos").join(folder);

        let files = {
            match fs::read_dir(&full_path) {
                Ok(files) => files,
                Err(e) => {
                    errors.push(e);
                    return (dir_ast, errors);
                }
            }
        };
        
        for file in files {
            if let Err(e) = file {
                errors.push(e);
                continue;
            }

            let path = PathBuf::from(file.unwrap().file_name());
            let stringified = path.to_str().unwrap_or("unknown");

            if let Some(ext) = path.extension() {
                let source = fs::read_to_string(&full_path.join(&path));

                if let Err(e) = source {
                    errors.push(e);
                    continue;
                }

                let ast = {
                    if let Some(parse) = self.language_layers.get(ext.to_str().unwrap()) {
                        parse(source.unwrap(), Rc::from(stringified))
                    } else {
                        continue; // skip unknown extensions
                    }
                };

                if let Err(e) = ast {
                    errors.extend(e.into_iter().map(|x| Error::other(x)));
                    continue;
                }

                dir_ast.extend(ast.unwrap());
            }
        }

        (dir_ast, errors)
    }

    fn init_gui_algos(&mut self) {
        self.gui.rotations = self.rotations.keys().map(|x| Rc::clone(&x)).collect();
        self.gui.rotations.sort();
        self.gui.pivot_selections = self.pivot_selections.keys().map(|x| Rc::clone(&x)).collect();
        self.gui.pivot_selections.sort();
        self.gui.distributions = self.distributions.keys().map(|x| Rc::clone(&x)).collect();
        self.gui.distributions.sort();
        self.gui.shuffles = self.shuffles.keys().map(|x| Rc::clone(&x)).collect();
        self.gui.shuffles.sort();
        self.gui.categories = self.categories.iter().map(|x| Rc::clone(x)).collect();
        self.gui.visuals = self.visuals.iter().map(|x| Rc::from(x.name())).collect();
        self.gui.sounds = self.sounds.iter().map(|x| Rc::from(x.name())).collect();
        self.gui.sorts = self.sorts.iter().map(
            |(category, sorts)| {
                let mut sorts_vec: Vec<Rc<str>> = sorts.keys().map(|x| Rc::clone(&x)).collect();
                sorts_vec.sort();
                (Rc::clone(category), sorts_vec)
            }
        ).collect();
    }

    fn try_load_automation(&mut self, name: &str) -> Result<Option<AutomationFile>, Error> {
        let full_path = program_dir!().join("automations").join(name);
        if full_path.exists() {
            let mut source = String::new();
            let mut f = File::open(full_path)?;
            f.read_to_string(&mut source)?;
            Ok(Some(AutomationFile::new(source.into(), Rc::from(name))))
        } else {
            Ok(None)
        }
    }

    fn reset_shuffle_automation(&mut self) {
        self.shuffle_automation.take();
    }

    fn init_automation_shuffle(&mut self) {
        let shuffle = Shuffle {
            name: Rc::from("Run automation"),
            callable: NativeCallable::new(
                Rc::new(|univ, _args, _task| {
                    if univ.shuffle_automation.get().is_none() {
                        let automations = univ.list_automations()?;

                        if univ.check_no_automations(&automations)? {
                            return Err(univ.vm.create_exception(UniLValue::String(
                                Rc::from("No automation available")
                            )));
                        }

                        univ.gui.build_fn = Gui::automation_selection;
                        univ.gui.automation_selection.set("Automation shuffle", automations.clone(), 0).unwrap();
                        univ.run_gui()?;

                        if univ.gui.automation_selection.back {
                            return Err(ExecutionInterrupt::StopAlgorithm);
                        }

                        let automation = {
                            if let Some(automation) =
                                univ.try_load_automation(&automations[univ.gui.automation_selection.index].filename)
                                    .map_err(|e| univ.automation_interpreter.create_exception(e.to_string().into()))?
                            {
                                automation
                            } else {
                                return Err(univ.vm.create_exception(UniLValue::String(
                                    Rc::from("The selected automation was deleted")
                                )));
                            }
                        };

                        univ.shuffle_automation.set(automation).unwrap();
                    }

                    let automation = get_expect!(univ.shuffle_automation);
                    univ.execute_automation(
                        Rc::clone(&automation.source),
                        Rc::clone(&automation.filename)
                    )?;

                    Ok(UniLValue::Null)
                }),
                1
            ).into()
        };

        self.shuffles.insert(Rc::clone(&shuffle.name), shuffle);
    }

    #[cfg(not(feature = "lite"))]
    fn get_algos_ast(&mut self, errors: &mut Vec<Error>) -> Vec<Expression> {
        log!(TraceLogLevel::LOG_INFO, "Loading algorithms");

        let mut toplevel_ast = {
            match unil::parse(COMPILATION_HEADER.to_string(), Rc::from("header.uni")) {
                Ok(ast) => ast,
                Err(err) => {
                    errors.extend(err.into_iter().map(|x| Error::other(x)));
                    Vec::new()
                }
            }
        };

        for type_ in ["utils", "distributions", "pivotSelections", "rotations", "shuffles", "sorts"] {
            let (mut ast, mut e) = self.load_algo_folder(type_);
            toplevel_ast.append(&mut ast);
            errors.append(&mut e);
        }

        toplevel_ast
    }

    #[cfg(not(feature = "lite"))]
    pub fn generate_headers(globals: &HashMap<Rc<str>, UniLType>) -> Result<(), Error> {
        let headers = program_dir!().join("headers");
        if !headers.exists() {
            fs::create_dir(&headers)?;
        }
        
        language_layers::generate_headers(globals)?;
        Ok(())
    }

    #[cfg(not(feature = "lite"))]
    fn compile_algos(&mut self, errors: &mut Vec<Error>, headers: bool) -> Option<Bytecode> {
        let mut toplevel_ast = self.get_algos_ast(errors);
        unil::swap_recognition::process(&mut toplevel_ast);

        match compiler::compile_and_get_globals(&toplevel_ast, &self.vm.globals.borrow()) {
            Ok((bytecode, globals)) => {
                if headers {
                    if let Err(e) = Self::generate_headers(globals.borrow().get_locals()) {
                        errors.push(e);
                    }
                }

                Some(bytecode)
            }
            Err(err) => {
                errors.extend(err.into_iter().map(|x| Error::other(x)));
                None
            }
        }
    }

    fn load_algos(&mut self) -> Result<bool, Vec<Error>> {
        let mut errors = Vec::new();

        #[cfg(not(feature = "lite"))]
        let bytecode = self.compile_algos(&mut errors, true);

        #[cfg(feature = "lite")]
        let bytecode: Option<Bytecode> = Some(
            decode_from_slice(BYTECODE, bincode::config::standard())
                .expect("Malformed bytecode").0
        );

        if let Some(bytecode) = bytecode {
            self.vm.set_bytecode(bytecode);
            let mut task = Task::new(0, &Rc::clone(&self.vm.globals));
            task.started = true;
            self.vm.schedule(task);

            if let Err(err) = self.execute() {
                match err {
                    ExecutionInterrupt::Quit => return Ok(true),
                    ExecutionInterrupt::Exception { value, traceback, thread } => {
                        errors.push(Error::other(format_traceback!(traceback, value, thread)));
                    }
                    _ => unreachable!(),
                }
            }
        }

        let sort_amt: usize = self.sorts.values().map(|x| x.len()).sum();
        self.categories.sort();

        log!(TraceLogLevel::LOG_INFO, "Loaded {} distributions", self.distributions.len());
        log!(TraceLogLevel::LOG_INFO, "Loaded {} pivot selections", self.pivot_selections.len());
        log!(TraceLogLevel::LOG_INFO, "Loaded {} rotations", self.rotations.len());
        log!(TraceLogLevel::LOG_INFO, "Loaded {} shuffles", self.shuffles.len());
        log!(TraceLogLevel::LOG_INFO, "Loaded {} sorts", sort_amt);
        self.init_automation_shuffle();
        self.init_gui_algos();

        match self.try_load_automation(RUN_ALL_SORTS_FILENAME) {
            Err(e) => errors.push(e),
            Ok(automation) => {
                if let Some(automation) = automation {
                    let _ = self.run_all_sorts.set(automation);
                }
            }
        }

        match self.try_load_automation(RUN_ALL_SHUFFLES_FILENAME) {
            Err(e) => errors.push(e),
            Ok(automation) => {
                if let Some(automation) = automation {
                    let _ = self.run_all_shuffles.set(automation);
                }
            }
        }

        if errors.is_empty() {
            Ok(false)
        } else {
            Err(errors)
        }
    }

    fn compute_font_size(&mut self) {
        self.font_size = (
            (
                get_expect!(self.rl_handle).get_screen_width() as f64 / 1280.0 +
                get_expect!(self.rl_handle).get_screen_height() as f64 / 720.0
            ) * 11.0
        ).round() as usize;
    }

    fn init_gui(&mut self) {
        log!(TraceLogLevel::LOG_INFO, "Initializing GUI");
        self.gui.init(get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread));
    }

    fn init_visuals(&mut self) -> Result<(), ExecutionInterrupt> {
        log!(TraceLogLevel::LOG_INFO, "Initializing visuals");

        let rl = get_expect_mut!(self.rl_handle);
        let thread = get_expect!(self.rl_thread);
        for visual in self.visuals.iter_mut() {
            visual.init(&self.shared, &mut self.gui, rl, thread)?;
        }

        Ok(())
    }

    fn set_window_size(&mut self) -> Result<(), ExecutionInterrupt> {
        {
            let rl = get_expect_mut!(self.rl_handle);
            rl.set_window_size(
                self.settings.resolution[0] as i32,
                self.settings.resolution[1] as i32
            );

            let _ = rl.begin_drawing(get_expect!(self.rl_thread)); // without this, get_screen_height() and get_screen_width() return the old value
        }
        
        self.compute_font_size();
        self.init_gui();
        self.loading_message("Preparing...")?;

        log!(TraceLogLevel::LOG_INFO, "Loading render texture");

        {
            let rl = get_expect_mut!(self.rl_handle);

            let width = rl.get_screen_width() as u32;
            let height = rl.get_screen_height() as u32;

            let _ = self.render_texture.set(
                rl.load_render_texture(get_expect!(self.rl_thread), width, height)
                    .expect("Could not load render texture")
            );
        }

        log!(TraceLogLevel::LOG_INFO, "Loading font");

        {
            let rl = get_expect_mut!(self.rl_handle);

            if let Some(x) = self.font.take() {
                rl.unload_font(x);
            }

            match rl.load_font_from_memory(
                get_expect!(self.rl_thread),
                FONT_FORMAT, DEFAULT_FONT_DATA,
                self.font_size as i32, None
            ) {
                Ok(font) => self.font.set(font.make_weak()).unwrap(),
                Err(e) => {
                    return Err(self.vm.create_exception(UniLValue::String(format!("Could not load font:\n{}", e).into())));
                }
            }
        }


        self.init_visuals()?;
        Ok(())
    }

    fn init_audio(&mut self) {
        if self.audio.get().is_none() {
            log!(TraceLogLevel::LOG_INFO, "Initializing audio");
            match OutputStream::try_default() {
                Ok((_stream, handle)) => {
                    self.audio.set(Audio { _stream, handle }).expect("Audio OnceCell was already set");
                    log!(TraceLogLevel::LOG_INFO, "Audio was initialized successfully");
                }
                Err(e) => {
                    log!(TraceLogLevel::LOG_ERROR, "Could not initialize audio");
                    log!(TraceLogLevel::LOG_ERROR, "    > {}", e);
                    log!(TraceLogLevel::LOG_WARNING, "Proceeding with no audio");
                }
            }
        }
    }

    pub fn loading_message(&mut self, msg: &str) -> Result<(), ExecutionInterrupt> {
        self.gui.build_fn = Gui::loading_panel;
        self.gui.loading_panel.set_message(msg);
        self.gui.draw_once(get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread))
    }

    pub fn init_run(&mut self) {
        let (handle, thread) = raylib::init()
            .size(self.settings.resolution[0] as i32, self.settings.resolution[1] as i32)
            .title(format!(
                "UniV {}v{}",
                if cfg!(feature = "lite") { "lite " } else { "" },
                VERSION
            ).as_str())
            .log_level(LOG_LEVEL)
            .build();

        self.rl_handle.set(handle).expect("Raylib handle OnceCell was already set");
        self.rl_thread.set(thread).expect("Raylib thread OnceCell was already set");

        log!(TraceLogLevel::LOG_INFO, "Loading icon");
        match Image::load_image_from_mem(LOGO_FORMAT, LOGO) {
            Ok(mut image) => {
                image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
                get_expect_mut!(self.rl_handle).set_window_icon(image);
            }
            Err(e) => {
                log!(TraceLogLevel::LOG_ERROR, "Could not load icon");
                log!(TraceLogLevel::LOG_ERROR, "    > {}", e);
                log!(TraceLogLevel::LOG_WARNING, "Proceeding with no icon");
            }
        }

        log!(TraceLogLevel::LOG_INFO, "Loading visual styles");
        self.visuals = visuals::load();
        self.visuals.sort_by(|a, b| a.name().cmp(b.name()));
        log!(TraceLogLevel::LOG_INFO, "Loaded {} visuals", self.visuals.len());

        log!(TraceLogLevel::LOG_INFO, "Loading sound engines");
        self.sounds = sounds::load();
        self.sounds.sort_by(|a, b| a.name().cmp(b.name()));
        log!(TraceLogLevel::LOG_INFO, "Loaded {} sounds", self.sounds.len());

        self.load_settings(true);

        if let Err(e) = self.set_window_size() {
            match e {
                ExecutionInterrupt::Quit => (),
                ExecutionInterrupt::Exception { value, traceback, thread } => {
                    let formatted = format_traceback!(traceback, value, thread);
                    log!(TraceLogLevel::LOG_ERROR, "{}", formatted.as_str());

                    self.gui.build_fn = Gui::popup;
                    self.gui.popup.set("Error", formatted.as_str()).unwrap();
                    let _ = self.run_gui();
                }
                _ => unreachable!()
            }

            return;
        }

        log!(TraceLogLevel::LOG_INFO, "Initializing sound engine");
        if let Err(e) = self.set_sound_by_name(&self.settings.sound.clone()) {
            if matches!(e, ExecutionInterrupt::Quit) {
                return;
            }

            log!(TraceLogLevel::LOG_ERROR, "Could not initialize sound engine");
            log!(TraceLogLevel::LOG_ERROR, "    > {}", expect_traceback!(e));

            if let Ok(id) = self.sounds.binary_search_by(|x| x.name().cmp("Default")) {
                log!(TraceLogLevel::LOG_WARNING, "Proceeding with default engine");

                if self.set_sound_by_id(id).is_err() {
                    return;
                }
            } else {
                log!(TraceLogLevel::LOG_ERROR, "Default engine is not available");
                log!(TraceLogLevel::LOG_WARNING, "Proceeding with first available engine");

                if self.set_sound_by_id(0).is_err() {
                    return;
                }
            }
        }

        if self.loading_message("Loading algorithms...").is_err() {
            return;
        }

        match self.load_algos() {
            Ok(quit) => {
                if quit {
                    return;
                }
            }
            Err(errors) => {
                let error_buf = report_errors("Something went wrong while loading algorithms", &errors);
                self.gui.build_fn = Gui::popup;
                self.gui.popup.set("Error", &error_buf).unwrap();
                if self.run_gui().is_err() {
                    return;
                }
            }
        }

        self.run();
    }

    fn reload_algos(&mut self) -> Result<(), ExecutionInterrupt> {
        self.loading_message("Unloading algorithms...")?;

        self.vm.reset();
        self.vm.reload_globals();
        self.distributions.clear();
        self.shuffles.clear();
        self.sorts.clear();
        self.categories.clear();
        self.pivot_selections.clear();
        self.rotations.clear();

        self.loading_message("Reloading algorithms...")?;

        match self.load_algos() {
            Ok(quit) => {
                if quit {
                    return Err(ExecutionInterrupt::Quit);
                }
            }
            Err(errors) => {
                let error_buf = report_errors("Something went wrong while loading algorithms", &errors);
                self.gui.build_fn = Gui::popup;
                self.gui.popup.set("Error", &error_buf).unwrap();
                self.run_gui()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn should_close(&self) -> bool {
        get_expect!(self.rl_handle).window_should_close()
    }

    fn enable_render_mode(&mut self) {
        self.render.active = true;
        self.render_fn = Self::rendered_highlight;
        self.sweep_frame_fn = Self::sweep_rendered_frame;
        self.shared.fps = self.settings.render_fps as u32;
    }

    fn enable_realtime_mode(&mut self) {
        self.render.active = false;
        self.render_fn = Self::realtime_highlight;
        self.sweep_frame_fn = Self::sweep_realtime_frame;
    }

    fn close_ffmpeg(&mut self) {
        if let Some(mut ffmpeg) = self.render.ffmpeg.take() {
            ffmpeg.video.stdin.take();
            ffmpeg.audio.stdin.take();
            let _ = ffmpeg.video.wait();
            let _ = ffmpeg.audio.wait();
        }
    }

    fn finalize_render(&mut self) -> Result<(), ExecutionInterrupt> {
        if let Some(mut ffmpeg) = self.render.ffmpeg.take() {
            // close ffmpeg pipes
            ffmpeg.video.stdin.take();
            ffmpeg.audio.stdin.take();

            ffmpeg.video.wait()
                .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?;
            ffmpeg.audio.wait()
                .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?;

            let thread_running = Arc::new(AtomicBool::new(true));
            let local_running = Arc::clone(&thread_running);
            let executable = self.render.ffmpeg_executable.clone();

            let handle = std::thread::spawn(move || {
                let res = Command::new(executable)
                    .args([
                        "-i", "tmp.mp4",
                        "-i", "tmp.wav",
                        "-map", "0:v",
                        "-map", "1:a",

                        "-c:v", "copy",
                        "-y", "output.mp4",
                    ])
                    .status();

                thread_running.store(false, atomic::Ordering::Relaxed);
                res
            });

            self.gui.build_fn = Gui::loading_panel;
            self.gui.loading_panel.set_message("Merging video and sound tracks...");
            self.gui.loading_panel.running = local_running;
            self.run_gui()?;

            let merge_result = handle.join()
                .map_err(|_| self.vm.create_exception(UniLValue::String(Rc::from("Error joining ffmpeg process"))))?
                .map_err(|e| self.vm.create_exception(UniLValue::String(e.to_string().into())))?;

            let _ = fs::remove_file("tmp.mp4");
            let _ = fs::remove_file("tmp.wav");

            if !merge_result.success() {
                return Err(self.vm.create_exception(UniLValue::String(Rc::from("Final video/audio merge process failed"))));
            }
        }

        Ok(())
    }

    fn save_background(&mut self) {
        let rl = get_expect_mut!(self.rl_handle);
        rl.set_trace_log(TraceLogLevel::LOG_NONE);

        if self.render.active {
            let mut draw = rl.begin_texture_mode(
                get_expect!(self.rl_thread), get_expect_mut!(self.gui.background)
            );

            draw.draw_texture(get_expect!(self.render_texture), 0, 0, Color::WHITE);
        } else {
            let width  = rl.get_screen_width();
            let height = rl.get_screen_height();

            let frame = rl.load_image_from_screen(get_expect!(self.rl_thread));
            let texture = rl.load_texture_from_image(get_expect!(self.rl_thread), &frame)
                .expect("Could not load texture from screen image");

            let mut draw = rl.begin_texture_mode(
                get_expect!(self.rl_thread), get_expect_mut!(self.gui.background)
            );

            draw_flipped_texture!(draw, &texture, width, height);
        }

        rl.set_trace_log(LOG_LEVEL);
    }

    #[inline]
    pub fn run_gui(&mut self) -> Result<(), ExecutionInterrupt> {
        self.gui.run(get_expect_mut!(self.rl_handle), get_expect!(self.rl_thread))
    }

    fn run_sorting_sequence(
        &mut self, distribution_id: i32, shuffle_id: i32, category_id: i32, sort_id: i32,
        length: usize, unique: usize, speed: f64
    ) -> Result<(), ExecutionInterrupt> {
        self.run_distribution(
            &Rc::clone(&self.gui.distributions[distribution_id as usize]),
            length,
            unique
        )?;

        self.run_shuffle(&Rc::clone(&self.gui.shuffles[shuffle_id as usize]))?;

        self.set_speed(speed)
            .map_err(|e| self.vm.create_exception(UniLValue::String(e)))?;

        let category = Rc::clone(&self.categories[category_id as usize]);
        self.run_sort(&category, &Rc::clone(&self.gui.sorts[&category][sort_id as usize]))
    }

    fn wrapped_runall_sequence(
        &mut self, distribution_id: i32, shuffle_id: i32, category_id: i32, sort_id: i32,
        length: usize, unique: usize, mut speed: f64, all_sorts: bool,
    ) -> Result<(), ExecutionInterrupt> {
        let distribution = Rc::clone(&self.gui.distributions[distribution_id as usize]);
        let shuffle = Rc::clone(&self.gui.shuffles[shuffle_id as usize]);

        if all_sorts {
            if self.user_values.is_empty() {
                self.store_user_values = true;
            } else {
                for value in self.user_values.iter().rev() {
                    self.autovalues.push_front(value.clone());
                }
            }
        }

        if let Err(e) = self.run_distribution(&distribution, length, unique) {
            if all_sorts {
                self.user_values.clear();
                self.store_user_values = false;
            }

            return Err(e);
        }

        if let Err(e) = self.run_shuffle(&shuffle) {
            if all_sorts {
                self.user_values.clear();
                self.store_user_values = false;
            }

            return Err(e);
        }

        if all_sorts {
            self.store_user_values = false;
        }

        let category = Rc::clone(&self.categories[category_id as usize]);
        let sort = Rc::clone(&self.gui.sorts[&category][sort_id as usize]);

        if !self.sorts.contains_key(&category) {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Unknown sort category \"{}\"", category
            ).into())));
        }

        if !self.sorts[&category].contains_key(&sort) {
            return Err(self.vm.create_exception(UniLValue::String(format!(
                "Unknown sort \"{}\"", sort
            ).into())));
        }

        if let UniLValue::Object(killers_obj) = &self.sorts[&category][&sort].killers {
            if let AnyObject::AnonObject(fields) = &*killers_obj.borrow() {
                if let Some(value) = fields.fields.get(&distribution) {
                    if let UniLValue::Object(obj) = value {
                        if let AnyObject::List(list) = &*obj.borrow() {
                            if list.contains(&UniLValue::String(shuffle)) {
                                speed = KILLER_INPUT_SPEED;
                            }
                        } else {
                            speed = KILLER_INPUT_SPEED;
                        }
                    } else {
                        speed = KILLER_INPUT_SPEED;
                    }
                }
            }
        }

        self.set_speed(speed)
            .map_err(|e| self.vm.create_exception(UniLValue::String(e)))?;

        if !all_sorts { // !all_sorts is all_shuffles
            if self.user_values.is_empty() {
                self.store_user_values = true;
            } else {
                for value in self.user_values.iter().rev() {
                    self.autovalues.push_front(value.clone());
                }
            }
        }

        if let Err(e) = self.run_sort(&category, &sort) {
            if !all_sorts {
                self.user_values.clear();
                self.store_user_values = false;
            }

            return Err(e);
        }

        if !all_sorts {
            self.store_user_values = false;
        }

        Ok(())
    }

    pub fn runall_sorting_sequence(
        &mut self, distribution_id: i32, shuffle_id: i32, category_id: i32, sort_id: i32,
        length: usize, unique: usize, speed: f64, all_sorts: bool
    ) -> Result<(), ExecutionInterrupt> {
        if let Err(e) = self.wrapped_runall_sequence(
            distribution_id, shuffle_id, category_id, sort_id,
            length, unique, speed, all_sorts
        ) {
            self.reset_autovalues();

            if let ExecutionInterrupt::Exception { value, traceback, thread } = e {
                let formatted = format_traceback!(traceback, value, thread);
                log!(TraceLogLevel::LOG_ERROR, "{}", formatted);

                self.save_background();
                self.gui.build_fn = Gui::popup;
                self.gui.popup.set("Error", formatted.as_str()).unwrap();
                self.run_gui()?;
                self.reset();
            } else if matches!(e, ExecutionInterrupt::StopAlgorithm) {
                return Err(e); // propagate stopalgorithm so that it stops the entire sequence
            }
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.keep_empty_frames = false;
        self.reset_speed();
        self.marks.clear();
        self.highlights.clear();
        self.reset_aux();
        self.reset_heatmaps();
    }

    fn stop_algorithm(&mut self) -> Result<(), ExecutionInterrupt> {
        self.reset();

        self.save_background();
        self.gui.build_fn = Gui::popup;
        self.gui.popup.set("Done", "Algorithm stopped.").unwrap();
        self.run_gui()
    }

    fn list_automations(&mut self) -> Result<Vec<AutomationFileInfo>, ExecutionInterrupt> {
        let mut errors = Vec::new();
        let mut output = Vec::new();

        let full_path = program_dir!().join("automations");

        match fs::read_dir(&full_path) {
            Err(e) => errors.push(e),
            Ok(dir) => {
                for file in dir {
                    if let Err(e) = file {
                        errors.push(e);
                        continue;
                    }

                    let path = PathBuf::from(file.unwrap().file_name());
                    let stringified = path.to_str().unwrap_or("unknown");

                    if let Some(ext) = path.extension() {
                        if ext.to_str().unwrap() != "ual" {
                            continue;
                        }

                        let source = fs::read_to_string(&full_path.join(&path));

                        if let Err(e) = source {
                            errors.push(e);
                            continue;
                        }

                        let filename = Rc::from(stringified);
                        let description = self.get_automation_description(
                            source.unwrap().into(),
                            Rc::clone(&filename)
                        );

                        if let Err(ExecutionInterrupt::Exception { value, traceback, thread }) = description {
                            errors.push(Error::other(format_traceback!(traceback, value, thread)));
                            continue;
                        }

                        output.push(AutomationFileInfo::new(filename, description.unwrap()));
                    }
                }
            }
        }

        if !errors.is_empty() {
            let mut error_buf = String::from("Something went wrong while loading automations:");

            for error in errors {
                error_buf.push('\n');
                error_buf.push_str(&error.to_string());
            }

            log!(TraceLogLevel::LOG_ERROR, "{}", error_buf);

            self.gui.build_fn = Gui::popup;
            self.gui.popup.set("Error", &error_buf).unwrap();
            self.run_gui()?;
        }

        Ok(output)
    }

    fn check_no_automations(&mut self, automations: &[AutomationFileInfo]) -> Result<bool, ExecutionInterrupt> {
        if automations.is_empty() {
            self.gui.build_fn = Gui::popup;
            self.gui.popup.set("Error", "No automation is loaded. This functionality is not available").unwrap();
            self.run_gui()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn check_no_algos(&mut self) -> Result<bool, ExecutionInterrupt> {
        if self.distributions.is_empty() || self.categories.is_empty() {
            self.gui.build_fn = Gui::popup;
            self.gui.popup.set("Error", "No algorithm is loaded. This functionality is not available").unwrap();
            self.run_gui()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[cfg(not(feature = "dev"))]
    fn find_or_install_ffmpeg(&mut self) -> Result<bool, ExecutionInterrupt> {
        for (msg, command) in [
            ("in program path", program_dir!().join("ffmpeg")), 
            ("globally", PathBuf::from("ffmpeg"))
        ] {
            log!(TraceLogLevel::LOG_INFO, "Attempting to find ffmpeg executable {}", msg);

            match Command::new(&command).arg("-version").output() {
                Ok(output) => {
                    if output.status.success() {
                        log!(TraceLogLevel::LOG_INFO, "Found ffmpeg at {:?}", command);
                        self.render.ffmpeg_executable = command;
                        return Ok(true);
                    }

                    if let Some(code) = output.status.code() {
                        log!(TraceLogLevel::LOG_WARNING, "ffmpeg command exited with code {}", code);
                    } else {
                        log!(TraceLogLevel::LOG_WARNING, "ffmpeg command exited with a non-zero exit code");
                    }
                }
                Err(e) => {
                    log!(TraceLogLevel::LOG_WARNING, "Failed to execute ffmpeg command: {}", e.to_string());
                }
            }
        }

        if ffmpeg::URL == "" {
            self.gui.build_fn = Gui::popup;
            self.gui.popup.set(
                "Error", 
                concat!(
                    "ffmpeg is not installed on your machine and it cannot be downloaded automatically for your platform.\n",
                    "Please install it manually if you want to use render mode"
                )
            ).unwrap();
            self.run_gui()?;
            return Ok(false);
        }

        self.gui.build_fn = Gui::selection;
        self.gui.selection.set(
            "Error", 
            concat!(
                "You enabled render mode, but it looks like ffmpeg is not present on your machine.\n",
                "Would you like to download it?"
            ),
            [
                "Yes",
                "No"
            ].into_iter().map(|x| Rc::from(x)).collect(), 
            0
        ).unwrap();
        self.run_gui()?;

        if self.gui.selection.index == 1 {
            return Ok(false);
        }

        let thread_running = Arc::new(AtomicBool::new(true));
        let local_running = Arc::clone(&thread_running);

        let handle = std::thread::spawn(move || {
            let inner = || {
                let client = reqwest::blocking::ClientBuilder::new()
                    .timeout(ffmpeg::DOWNLOAD_TIMEOUT)
                    .build()
                    .map_err(|e| e.to_string())?;

                let response = client.get(ffmpeg::URL).send()
                    .map_err(|e| e.to_string())?;

                let status = response.status();
                if !status.is_success() {
                    return Err({
                        if let Some(reason) = status.canonical_reason() {
                            format!("{}: {}", status.as_str(), reason)
                        } else {
                            status.to_string()
                        }
                    });
                }

                let data = response.bytes()
                    .map_err(|e| e.to_string())?
                    .to_vec();

                let mut gz = flate2::read::GzDecoder::new(&data[..]);
                let mut decoded_data = Vec::with_capacity(data.len());
                gz.read_to_end(&mut decoded_data)
                    .map_err(|e| e.to_string())?;

                drop(gz);
                drop(data);

                let output_file = program_dir!().join({
                    if cfg!(windows) {
                        "ffmpeg.exe"
                    } else {
                        "ffmpeg"
                    }
                });

                let mut f = File::create(&output_file)
                    .map_err(|e| e.to_string())?;
                f.write_all(&decoded_data)
                    .map_err(|e| e.to_string())?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    // make file readable and executable
                    fs::set_permissions(output_file, fs::Permissions::from_mode(0o555))
                        .map_err(|e| e.to_string())?;
                }

                Ok(())
            };

            let result = inner();
            thread_running.store(false, atomic::Ordering::Relaxed);
            result
        });

        self.gui.build_fn = Gui::loading_panel;
        self.gui.loading_panel.set_message("Downloading and installing ffmpeg...");
        self.gui.loading_panel.running = local_running;
        self.run_gui()?;

        handle.join()
            .map_err(|_| self.vm.create_exception(UniLValue::String(Rc::from("Error joining ffmpeg download process"))))?
            .map_err(|e| self.vm.create_exception(UniLValue::String(
                format!("An error occurred while trying to download ffmpeg:\n{}", e.to_string()
            ).into())))?;

        log!(TraceLogLevel::LOG_INFO, "Verifying ffmpeg installation");
        self.loading_message("Verifying ffmpeg installation...")?;

        let command = program_dir!().join("ffmpeg");
        let error = {
            match Command::new(&command).arg("-version").output() {
                Ok(output) => {
                    if output.status.success() {
                        log!(TraceLogLevel::LOG_INFO, "ffmpeg was installed correctly");
                        self.render.ffmpeg_executable = command;
                        return Ok(true);
                    }

                    if let Some(code) = output.status.code() {
                        log!(TraceLogLevel::LOG_ERROR, "ffmpeg command exited with code {}", code);
                        format!("exit code: {}", code)
                    } else {
                        log!(TraceLogLevel::LOG_ERROR, "ffmpeg command exited with a non-zero exit code");
                        String::from("non-zero exit code")
                    }
                }
                Err(e) => {
                    let error = e.to_string();
                    log!(TraceLogLevel::LOG_ERROR, "Failed to execute ffmpeg command: {}", error);
                    error
                }
            }
        };

        self.gui.build_fn = Gui::popup;
        self.gui.popup.set("Error", format!("ffmpeg was not installed correctly.\nError: {}", error).as_str()).unwrap();
        self.run_gui()?;
        Ok(false)
    }

    fn main_menu(&mut self) -> Result<(), ExecutionInterrupt> {
        loop {
            #[cfg(not(feature = "dev"))]
            if self.settings.render {
                if !self.render.active && !self.find_or_install_ffmpeg()? {
                    self.settings.render = false;
                    self.try_save_settings();
                    continue;
                }

                self.enable_render_mode();
            } else {
                self.enable_realtime_mode();

                if self.settings.play_sound {
                    self.init_audio();
                }
            }

            self.gui.build_fn = Gui::selection;
            self.gui.selection.set(
                "Welcome to UniV!",
                "",
                [
                    "Run sort",
                    "Run all sorts",
                    "Run all shuffles",
                    "Run automation",
                    "Settings",
                    "Quit"
                ].into_iter().map(|x| Rc::from(x)).collect(),
                0
            ).unwrap();
            self.run_gui()?;

            match self.gui.selection.index {
                0 => {
                    if self.check_no_algos()? {
                        continue;
                    }

                    loop {
                        self.reset_shuffle_automation();
                        self.gui.build_fn = Gui::run_sort;
                        self.run_gui()?;

                        if self.gui.run_sort.back {
                            break;
                        }

                        if self.gui.run_sort.speed <= 0.0 {
                            self.gui.build_fn = Gui::popup;
                            self.gui.popup.set("Error", "Speed cannot be less than or equal to 0").unwrap();
                            self.run_gui()?;
                            continue;
                        }

                        if self.gui.run_sort.unique_amt > self.gui.run_sort.array_length {
                            self.gui.build_fn = Gui::popup;
                            self.gui.popup.set("Error", "Unique amount cannot be greater than array length").unwrap();
                            self.run_gui()?;
                            continue;
                        }

                        self.set_visual_by_name(&Rc::clone(&self.gui.visuals[self.gui.run_sort.visual as usize]))
                            .expect("GUI returned invalid visual ID");

                        if let Err(e) = self.run_sorting_sequence(
                            self.gui.run_sort.distribution,
                            self.gui.run_sort.shuffle,
                            self.gui.run_sort.category,
                            self.gui.run_sort.sort,
                            self.gui.run_sort.array_length,
                            self.gui.run_sort.unique_amt,
                            self.gui.run_sort.speed
                        ) {
                            if matches!(e, ExecutionInterrupt::StopAlgorithm) {
                                self.stop_algorithm()?;
                                self.finalize_render()?;
                                continue;
                            } else {
                                return Err(e);
                            }
                        }

                        self.finalize_render()?;

                        self.gui.build_fn = Gui::selection;
                        self.gui.selection.set(
                            "Done!",
                            "Continue?",
                            [
                                "Yes",
                                "No"
                            ].into_iter().map(|x| Rc::from(x)).collect(),
                            0
                        ).unwrap();
                        self.run_gui()?;

                        if self.gui.selection.index == 1 {
                            break;
                        }
                    }
                }
                1 => {
                    if self.check_no_algos()? {
                        continue;
                    }

                    if self.run_all_sorts.get().is_none() {
                        self.gui.build_fn = Gui::popup;
                        self.gui.popup.set(
                            "Error",
                            format!("Could not find '{}' automation file", RUN_ALL_SORTS_FILENAME).as_str()
                        ).unwrap();
                        self.run_gui()?;
                        continue;
                    }

                    loop {
                        self.reset_shuffle_automation();
                        self.gui.build_fn = Gui::run_all_sorts;
                        self.run_gui()?;

                        if self.gui.run_all_sorts.back {
                            break;
                        }

                        if self.gui.run_all_sorts.length_mlt <= 0.0 {
                            self.gui.build_fn = Gui::popup;
                            self.gui.popup.set("Error", "Array length multiplier cannot be less than or equal to 0").unwrap();
                            self.run_gui()?;
                            continue;
                        }

                        if self.gui.run_all_sorts.unique_div < 1.0 {
                            self.gui.build_fn = Gui::popup;
                            self.gui.popup.set("Error", "Unique divisor cannot be less than 1").unwrap();
                            self.run_gui()?;
                            continue;
                        }

                        if self.gui.run_all_sorts.speed <= 0.0 {
                            self.gui.build_fn = Gui::popup;
                            self.gui.popup.set("Error", "Speed cannot be less than or equal to 0").unwrap();
                            self.run_gui()?;
                            continue;
                        }

                        self.set_visual_by_name(&Rc::clone(&self.gui.visuals[self.gui.run_all_sorts.visual as usize]))
                            .expect("GUI returned invalid visual ID");

                        self.automation_interpreter.reset();

                        if self.gui.run_all_sorts.category == 0 { // 0 is all categories
                            self.automation_interpreter.mode = AutomationMode::RunSorts;
                        } else {
                            let category = Rc::clone(&self.gui.categories[self.gui.run_all_sorts.category as usize - 1]);
                            self.automation_interpreter.mode = AutomationMode::RunCategory(category);
                        }

                        if let Err(e) = self.execute_automation(
                            Rc::clone(&get_expect!(self.run_all_sorts).source),
                            Rc::clone(&get_expect!(self.run_all_sorts).filename)
                        ) {
                            if matches!(e, ExecutionInterrupt::StopAlgorithm) {
                                self.stop_algorithm()?;
                                self.finalize_render()?;
                                self.user_values.clear();
                                continue;
                            } else {
                                return Err(e);
                            }
                        }

                        self.finalize_render()?;
                        self.user_values.clear();
                        self.gui.build_fn = Gui::popup;
                        self.gui.popup.set("Finished", "All sorts have been visualized").unwrap();
                        self.run_gui()?;
                    }
                }
                2 => {
                    if self.check_no_algos()? {
                        continue;
                    }

                    if self.run_all_shuffles.get().is_none() {
                        self.gui.build_fn = Gui::popup;
                        self.gui.popup.set(
                            "Error",
                            format!("Could not find '{}' automation file", RUN_ALL_SHUFFLES_FILENAME).as_str()
                        ).unwrap();
                        self.run_gui()?;
                        continue;
                    }

                    loop {
                        self.reset_shuffle_automation();
                        self.gui.build_fn = Gui::run_all_shuffles;
                        self.run_gui()?;

                        if self.gui.run_all_shuffles.back {
                            break;
                        }

                        if self.gui.run_all_shuffles.speed <= 0.0 {
                            self.gui.build_fn = Gui::popup;
                            self.gui.popup.set("Error", "Speed cannot be less than or equal to 0").unwrap();
                            self.run_gui()?;
                            continue;
                        }

                        if self.gui.run_all_shuffles.unique_amt > self.gui.run_all_shuffles.array_length {
                            self.gui.build_fn = Gui::popup;
                            self.gui.popup.set("Error", "Unique amount cannot be greater than array length").unwrap();
                            self.run_gui()?;
                            continue;
                        }

                        self.set_visual_by_name(&Rc::clone(&self.gui.visuals[self.gui.run_all_shuffles.visual as usize]))
                            .expect("GUI returned invalid visual ID");

                        self.automation_interpreter.reset();
                        self.automation_interpreter.mode = AutomationMode::RunShuffles;

                        if let Err(e) = self.execute_automation(
                            Rc::clone(&get_expect!(self.run_all_shuffles).source),
                            Rc::clone(&get_expect!(self.run_all_shuffles).filename)
                        ) {
                            if matches!(e, ExecutionInterrupt::StopAlgorithm) {
                                self.stop_algorithm()?;
                                self.finalize_render()?;
                                self.user_values.clear();
                                continue;
                            } else {
                                return Err(e);
                            }
                        }

                        self.finalize_render()?;
                        self.user_values.clear();
                        self.gui.build_fn = Gui::popup;
                        self.gui.popup.set("Finished", "All shuffles have been visualized").unwrap();
                        self.run_gui()?;
                    }
                }
                3 => {
                    let mut last_index = 0;
                    let automations = self.list_automations()?;

                    if self.check_no_automations(&automations)? {
                        continue;
                    }

                    loop {
                        self.reset_shuffle_automation();
                        self.gui.build_fn = Gui::automation_selection;
                        self.gui.automation_selection.set("Run automation", automations.clone(), last_index).unwrap();
                        self.run_gui()?;

                        if self.gui.automation_selection.back {
                            break;
                        }

                        last_index = self.gui.automation_selection.index;
                        let automation = {
                            if let Some(automation) =
                                self.try_load_automation(&automations[last_index].filename)
                                    .map_err(|e| self.automation_interpreter.create_exception(e.to_string().into()))?
                            {
                                automation
                            } else {
                                return Err(self.automation_interpreter.create_exception(
                                    Rc::from("The selected automation was deleted")
                                ));
                            }
                        };

                        if let Err(e) = self.execute_automation(automation.source, automation.filename) {
                            if matches!(e, ExecutionInterrupt::StopAlgorithm) {
                                self.stop_algorithm()?;
                                self.finalize_render()?;
                                continue;
                            } else {
                                return Err(e);
                            }
                        }

                        self.finalize_render()?;
                        self.gui.build_fn = Gui::popup;
                        self.gui.popup.set("Finished", "Automation complete").unwrap();
                        self.run_gui()?;
                    }
                }
                4 => {
                    self.gui.settings.load(&self.settings, &self.gui.sounds);

                    loop {
                        self.gui.build_fn = Gui::settings;
                        self.gui.settings.partial_reset();
                        self.run_gui()?;

                        let new_res = self.settings.resolution != self.gui.settings.object.resolution;

                        if self.gui.settings.config_deleted &&
                            (self.gui.settings.back || !new_res) &&
                            !(self.gui.settings.sound_setup || self.gui.settings.reload_algos)
                        {
                            // if a configuration is deleted, reinitialize visuals so that they can reload their configs
                            // (and potentially start the configuration screen). if `new_res` is true and we're saving,
                            // this is not needed because `set_window_size` will be called and do it on its own.
                            // also, don't do this if the user pressed on the sound setup or reload algorithms button,
                            // because that would be confusing
                            self.init_visuals()?;
                        }

                        if !self.gui.settings.back {
                            if self.gui.settings.sound_setup {
                                let sound = self.sounds.get_mut(self.gui.settings.curr_sound)
                                    .expect("GUI returned invalid sound ID");

                                sound.prepare(
                                    &self.shared,
                                    &mut self.gui,
                                    get_expect_mut!(self.rl_handle),
                                    get_expect!(self.rl_thread)
                                )?;

                                self.gui.settings.load_configs();
                                continue;
                            }

                            if cfg!(not(feature = "lite")) && self.gui.settings.reload_algos {
                                self.reload_algos()?;
                                continue;
                            }

                            if self.curr_sound != self.gui.settings.curr_sound {
                                self.set_sound_by_id(self.gui.settings.curr_sound)
                                    .expect("GUI returned invalid sound ID");
                            }

                            let new_reverb  = self.settings.reverb  != self.gui.settings.object.reverb;
                            let new_profile = self.settings.profile != self.gui.settings.object.profile;

                            if self.settings != self.gui.settings.object {
                                self.settings = self.gui.settings.object.clone();
                                self.try_save_settings();
                            }

                            if new_res {
                                self.set_window_size()?;
                            }

                            if new_profile {
                                self.try_load_current_profile();
                            }

                            if new_reverb {
                                self.sync_reverb();
                                get_sound!(self).prepare(
                                    &self.shared,
                                    &mut self.gui,
                                    get_expect_mut!(self.rl_handle),
                                    get_expect!(self.rl_thread)
                                )?;
                            }
                        }

                        break;
                    }
                }
                5 => break,
                _ => unreachable!()
            }
        }

        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.main_menu() {
                match e {
                    ExecutionInterrupt::Quit => break,
                    ExecutionInterrupt::Exception { value, traceback, thread } => {
                        let formatted = format_traceback!(traceback, value, thread);
                        log!(TraceLogLevel::LOG_ERROR, "{}", formatted);

                        self.save_background();
                        self.gui.build_fn = Gui::popup;
                        self.gui.popup.set("Error", formatted.as_str()).unwrap();
                        if self.run_gui().is_err() {
                            break;
                        }

                        self.reset();
                        self.close_ffmpeg(); // in case an exception occurs, close any existing connection to ffmpeg
                    }
                    _ => unreachable!()
                }
            } else {
                break;
            }
        }
    }
}

impl Drop for UniV {
    fn drop(&mut self) {
        self.close_ffmpeg();

        if let Some(mut rl) = self.rl_handle.take() {
            if let Some(font) = self.font.take() {
                rl.unload_font(font);
            }

            // without this, destructors of render textures may be called later than raylib's, causing a segfault

            let thread = get_expect!(self.rl_thread);
            for visual in self.visuals.iter_mut() {
                visual.unload(&mut rl, thread);
            }

            self.visuals.clear();
            self.gui.background.take();
            self.render_texture.take();
        }
    }
}

fn main() -> Result<(), Error> {
    let args_map: HashMap<String, usize> = env::args().enumerate().map(|(i, x)| (x, i)).collect();

    PROGRAM_DIR.set({
        match env::current_exe() {
            Ok(path) => {
                if args_map.contains_key("--debug") {
                    log!(TraceLogLevel::LOG_INFO, "Running from debugger");
                    PathBuf::new()
                } else if env::var("CARGO").is_ok() {
                    log!(TraceLogLevel::LOG_INFO, "Running from cargo");
                    PathBuf::new()
                } else if let Some(parent) = path.parent() {
                    PathBuf::from(parent)
                } else {
                    log!(TraceLogLevel::LOG_ERROR, "Could not fetch executable parent directory");
                    log!(TraceLogLevel::LOG_WARNING, "Falling back to current working directory");
                    PathBuf::new()
                }
            }
            Err(e) => {
                log!(TraceLogLevel::LOG_ERROR, "Could not fetch executable location");
                log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                log!(TraceLogLevel::LOG_WARNING, "Falling back to current working directory");
                PathBuf::new()
            }
        }
    }).expect("Program directory OnceCell was already set");

    #[cfg(feature = "dev")]
    {
        if args_map.contains_key("--compile-algos") {
            return dev::compile_algos(args_map.contains_key("--with-headers"));
        }

        if args_map.contains_key("--generate-headers") {
            return dev::generate_headers();
        }
    }

    #[cfg(not(feature = "dev"))] 
    {
        // avoids raylib unloading messages flood when the program panics
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            unsafe { raylib::ffi::SetTraceLogLevel(TraceLogLevel::LOG_NONE as i32) };
            default_hook(info);
        }));

        UniV::new().init_run();
    }

    Ok(())
}
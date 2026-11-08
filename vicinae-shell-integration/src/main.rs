use std::collections::HashMap;

use clap::{Parser, ValueEnum};
use fast_image_resize::Resizer;
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{
        Connection, globals::registry_queue_init, protocol::wl_pointer::WlPointer,
    },
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::{LayerShell, LayerSurface},
    shm::{Shm, slot::SlotPool},
};

#[derive(ValueEnum, Clone)]
enum CornerPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

#[derive(ValueEnum, Clone)]
enum FillType {
    Strecth,
    Fill,
}

#[derive(ValueEnum, Clone)]
enum BackgroundFilterType {
    Linear,
    Nearest,
}

mod smithay;
mod windows;

#[derive(Parser)]
#[command(name = "vicinae-shell-integration")]
#[command(author = "drewkri")]
#[command(version = "0.1.0")]
#[command(
    about = "Program for use with Vicinae and the Quick Menu extension to set up a small desk shell."
)]
struct Args {
    /// Provides an image path to use as the desktop background. If no image is provided,
    /// no desktop background layer will be created, so you are free to use another program like
    /// swaybg instead.
    #[arg(short, long)]
    background: Option<String>,

    /// Provides an image path to use as the Niri backdrop, shown behind the workspaces
    /// in the overview. If you set this argument, make sure you set a layer rule in your Niri
    /// configuration targeting this layer. Its namespace is "vicinae-shell-integration.backdrop"
    /// so you can do something like
    /// `layer-rule {
    ///     match namespace="vicinae-shell-integration.backdrop"
    ///     place-within-backdrop true
    /// }`
    #[arg(long)]
    backdrop: Option<String>,

    /// Width of the hot corner trigger area
    #[arg(short = 'W', long)]
    corner_width: Option<u32>,
    /// Height of the hot corner trigger area
    #[arg(short = 'H', long)]
    corner_height: Option<u32>,
    /// Corner to place the trigger area in
    #[arg(short, long, default_value = "top-right")]
    corner_position: CornerPosition,

    /// How background/backdrop images should fill the space
    #[arg(long, default_value = "fill")]
    fill_type: FillType,

    /// Filter to use when resizing background/backdrop images
    #[arg(long, default_value = "linear")]
    filter_type: BackgroundFilterType,
}

struct Window {
    surface: LayerSurface,
    window_type: WindowType,
    output_id: u32, // ID of the wl_output object this window is on.
}

#[derive(PartialEq)]
enum WindowType {
    HotCorner,
    DesktopBackground,
    OverviewBackdrop,
}

struct AppState {
    args: Args,

    compositor_state: CompositorState,
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    pointer: Option<WlPointer>,
    layer_shell: LayerShell,
    windows: HashMap<u32, Window>,
    shm: Shm,
    pool: SlotPool,

    resizer: Resizer,
    exit: bool,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor_state =
        CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");

    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");
    let pool = SlotPool::new(10 * 10 * 4, &shm).expect("unable to create shm pool");

    // Windows will be created for each output advertised, see OutputHandler in smithay.rs

    // Create the hot corner window

    let mut state = AppState {
        args,

        compositor_state,
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        layer_shell,
        pointer: None,
        windows: HashMap::new(),
        shm,
        pool,

        resizer: Resizer::new(),
        exit: false,
    };

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();

        // never actually set so this is a bit pointless but kept just in case this changes later.
        // maybe in the future this will have a dbus interface or something to change wallpapers
        // but you could also just kill and restart the process which is probably easier tbh
        if state.exit {
            println!("exiting example");
            break;
        }
    }
}

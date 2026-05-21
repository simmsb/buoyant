mod colour;
mod font;
mod state;
mod view;

use std::process::exit;
use std::time::{Duration, Instant};

use buoyant::app::{App, Harness};
use buoyant::event::{Event, Key, simulator::MouseTracker};
use buoyant::focus::{BoundaryBehavior, FocusAction, Role};
use buoyant::render_target::{EmbeddedGraphicsRenderTarget, RenderTarget as _};
use buoyant::{animation::Animation, match_view, view::prelude::*};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay, SimulatorEvent, Window};

use self::state::OperationState;

pub mod keys {
    use buoyant::event::Key;

    pub const UP_CLICK: Key = Key::UpArrow;
    pub const UP_HOLD: Key = Key::Character('1');
    pub const DOWN_CLICK: Key = Key::DownArrow;
    pub const DOWN_HOLD: Key = Key::Character('3');
    pub const CONFIRM_CLICK: Key = Key::Character('0');
    pub const CONFIRM_HOLD: Key = Key::Character('5');
    pub const POWER_CLICK: Key = Key::Character('6');
    pub const POWER_HOLD: Key = Key::Character('7');
}

pub mod ui {
    pub use crate::*;
}

const fn root_view_differ_size<V, T, S>(f: fn(T) -> V) -> usize
where
    V: ViewLayout<S>,
    V::Renderables: buoyant::render::Diffable,
{
    use buoyant::render::Diffable;

    V::Renderables::SIZE.div_ceil(8) + 1
}

fn root_view_renderables_name<V, T, S>(f: fn(T) -> V) -> &'static str
where
    V: ViewLayout<S>,
    V::Renderables: buoyant::render::Diffable,
{
    use buoyant::render::Diffable;

    std::any::type_name::<V::Renderables>()
}

fn pretty_print_format(input: &str) -> String {
    let mut result = String::new();
    let mut indent_level = 0;

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '<' | '(' | '[' => {
                result.push(c);
                result.push('\n');
                indent_level += 1;
                result.push_str(&"  ".repeat(indent_level));
            }
            '>' | ')' | ']' => {
                result.push('\n');
                indent_level = indent_level.saturating_sub(1);
                result.push_str(&"  ".repeat(indent_level));
                result.push(c);
            }
            ',' => {
                result.push(c);
                result.push('\n');
                result.push_str(&"  ".repeat(indent_level));
                // Skip spaces immediately following a comma
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            _ => result.push(c),
        }
    }
    result
}

fn main() {
    let size = Size::new(320, 480);
    let mut display: SimulatorDisplay<colour::ColorFormat> = SimulatorDisplay::new(size);
    let mut target = EmbeddedGraphicsRenderTarget::new_hinted(&mut display, colour::BLACK);
    let mut window = Window::new("Scooter", &OutputSettings::default());
    // Send at least one update to the window so it doesn't panic when fetching events
    window.update(target.display());

    let app_start = Instant::now();
    let mut touch_tracker = MouseTracker::new();

    // Create app with view lifecycle management
    let mut app = App::new(state::State::new(), size.into(), view::root_view)
        .with_roles(Role::Button | Role::Container);

    // Acquire initial focus
    app.focus_forward();

    let mut diffing_mem = [0u8; root_view_differ_size(view::root_view)];

    // Main event loop
    loop {
        // Sync app time with real wall clock time
        app.set_time(app_start.elapsed());

        // Collect and process simulator events
        window
            .events()
            .filter_map(|event| {
                if event == SimulatorEvent::Quit {
                    exit(0);
                }
                touch_tracker.process_event(event)
            })
            .for_each(|event| {
                app.send(event);
            });

        if let Some(action) = app.state().page_action {
            let current_page = app.state().page;
            let new_page = current_page.handle_action(action);
            let mut state = app.state_mut();

            if let Some(new_page) = new_page {
                state.page = new_page;
                state.page_action = None;
            }
        }

        if let Some(next_speed_mode) = app.state().next_speed_mode {
            if let OperationState::Active(a) = &mut app.state_mut().operation_state {
                a.speed_mode = next_speed_mode;
            }

            app.state_mut().next_speed_mode = None;
        }

        // Only render if active animation was reported or redraw needed
        if app.should_redraw() || target.clear_animation_status() {
            // n += 1;
            // Render animated transition between source and target trees
            // app.render_only_target(&mut target, &colour::ColorFormat::WHITE);
            app.render_animated_diffed(&mut target, &colour::ColorFormat::BLACK, &mut diffing_mem);

            // Draw focus overlay
            // app.draw_focus_overlay(&mut target, colour::ColorFormat::CSS_YELLOW, 1);

            // Send to the display
            window.update(target.display());
            // Clear for the next frame
            // target.clear(colour::ColorFormat::RED);
        } else {
            // limit polling for updates to ~30 fps when idle
            std::thread::sleep(Duration::from_millis(33));
        }
    }
}

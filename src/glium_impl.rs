use std::thread::sleep_ms;
use glium::Display;
use glutin::{Event, ElementState};
use clock_ticks::precise_time_ns;
use carboxyl::{Cell, Sink, Stream};
use input::Button;
use glutin_window::{map_key, map_mouse};

use button::{ButtonEvent, ButtonState};
use ::Window;


/// Glium implementation of an application loop.
pub struct GliumWindow {
    display: Display,
    tick_length: u64,
    tick_sink: Sink<u64>,
    winpos_sink: Sink<(i32, i32)>,
    winsize_sink: Sink<(u32, u32)>,
    button_sink: Sink<ButtonEvent>,
    mouse_motion_sink: Sink<(i32, i32)>,
    mouse_wheel_sink: Sink<i32>,
    focus_sink: Sink<bool>,
    char_sink: Sink<char>,
}

impl GliumWindow {
    /// Create a new Glium loop.
    ///
    /// # Parameters
    ///
    /// `tick_length` is the minimum duration of a tick in nanoseconds.
    pub fn new(display: Display, tick_length: u64) -> GliumWindow {
        GliumWindow {
            display: display,
            tick_length: tick_length,
            tick_sink: Sink::new(),
            button_sink: Sink::new(),
            mouse_motion_sink: Sink::new(),
            mouse_wheel_sink: Sink::new(),
            focus_sink: Sink::new(),
            winpos_sink: Sink::new(),
            winsize_sink: Sink::new(),
            char_sink: Sink::new(),
        }
    }

    fn dispatch(&self, event: Event) {
        fn to_button_state(state: ElementState) -> ButtonState {
            match state {
                ElementState::Pressed => ButtonState::Pressed,
                ElementState::Released => ButtonState::Released,
            }
        }

        match event {
            Event::KeyboardInput(state, _, Some(vkey)) =>
                self.button_sink.send(ButtonEvent {
                    button: Button::Keyboard(map_key(vkey)),
                    state: to_button_state(state),
                }),
            Event::MouseInput(state, button) =>
                self.button_sink.send(ButtonEvent {
                    button: Button::Mouse(map_mouse(button)),
                    state: to_button_state(state),
                }),
            Event::MouseWheel(a) => self.mouse_wheel_sink.send(a),
            Event::MouseMoved(a) => self.mouse_motion_sink.send(a),
            Event::Focused(a) => self.focus_sink.send(a),
            Event::Resized(w, h) => self.winsize_sink.send((w, h)),
            Event::Moved(x, y) => self.winpos_sink.send((x, y)),
            Event::ReceivedCharacter(c) => self.char_sink.send(c),
            _ => (),
        }
    }
}

impl Window for GliumWindow {
    fn ticks(&self) -> Stream<u64> {
        self.tick_sink.stream()
    }

    fn position(&self) -> Cell<(i32, i32)> {
        self.winpos_sink.stream().hold((0, 0))
    }

    fn size(&self) -> Cell<(u32, u32)> {
        self.winsize_sink.stream().hold((0, 0))
    }

    fn buttons(&self) -> Stream<ButtonEvent> {
        self.button_sink.stream()
    }

    fn characters(&self) -> Stream<char> {
        self.char_sink.stream()
    }

    fn cursor(&self) -> Cell<(i32, i32)> {
        self.mouse_motion_sink.stream().hold((0, 0))
    }

    fn wheel(&self) -> Cell<i32> {
        self.mouse_wheel_sink.stream().hold(0)
    }

    fn focus(&self) -> Cell<bool> {
        self.focus_sink.stream().hold(true)
    }

    fn start(&self) {
        let mut time = precise_time_ns();
        let mut next_tick = time;
        'main: loop {
            time = precise_time_ns();
            if time >= next_tick {
                let diff = time - next_tick;
                let delta = diff - diff % self.tick_length;
                next_tick += delta;
                for ev in self.display.poll_events() {
                    if let Event::Closed = ev { break 'main }
                    self.dispatch(ev);
                }
                self.tick_sink.send(delta);
                // Make sure that drawing is finished at the end of a tick
                self.display.synchronize();
            }
            else {
                sleep_ms((next_tick - time) as u32);
            }
        }
    }
}

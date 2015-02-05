use std::time::duration::Duration;
use std::old_io::timer::sleep;
use std::num::FromPrimitive;
use glium::Display;
use glutin::Event;
use clock_ticks::precise_time_ns;
use carboxyl::{Cell, Sink, Stream};
use input::Button;

use button::ButtonEvent;
use traits::ApplicationLoop;


/// Glium implementation of an application loop.
pub struct GliumLoop {
    display: Display,
    tick_length: u64,
    tick_sink: Sink<u64>,
    button_sink: Sink<ButtonEvent>,
    mouse_motion_sink: Sink<(i32, i32)>,
    mouse_wheel_sink: Sink<i32>,
    focus_sink: Sink<bool>,
}

impl GliumLoop {
    /// Create a new Glium loop.
    ///
    /// # Parameters
    ///
    /// `tick_length` is the minimum duration of a tick in nanoseconds.
    pub fn new(display: Display, tick_length: u64) -> GliumLoop {
        GliumLoop {
            display: display,
            tick_length: tick_length,
            tick_sink: Sink::new(),
            button_sink: Sink::new(),
            mouse_motion_sink: Sink::new(),
            mouse_wheel_sink: Sink::new(),
            focus_sink: Sink::new(),
        }
    }

    fn dispatch(&self, event: Event) {
        use button::ButtonState;
        use glutin::{self, ElementState};
        use input;

        fn to_button_state(state: ElementState) -> ButtonState {
            match state {
                ElementState::Pressed => ButtonState::Pressed,
                ElementState::Released => ButtonState::Released,
            }
        }

        match event {
            Event::KeyboardInput(state, code, _) =>
                match FromPrimitive::from_u8(code) {
                    Some(key) => self.button_sink.send(ButtonEvent {
                        button: Button::Keyboard(key),
                        state: to_button_state(state),
                    }),
                    None => (),
                },
            Event::MouseInput(state, button) =>
                self.button_sink.send(ButtonEvent {
                    button: Button::Mouse(match button {
                        glutin::MouseButton::LeftMouseButton =>
                            input::MouseButton::Left,
                        glutin::MouseButton::RightMouseButton =>
                            input::MouseButton::Right,
                        glutin::MouseButton::MiddleMouseButton =>
                            input::MouseButton::Middle,
                        glutin::MouseButton::OtherMouseButton(code) =>
                            FromPrimitive::from_u8(code).unwrap(),
                    }),
                    state: to_button_state(state),
                }),
            Event::MouseWheel(a) => self.mouse_wheel_sink.send(a),
            Event::MouseMoved(a) => self.mouse_motion_sink.send(a),
            Event::Focused(a) => self.focus_sink.send(a),
            // TODO: handle all events
            _ => (),
        }
    }
}

impl ApplicationLoop for GliumLoop {
    fn ticks(&self) -> Stream<u64> {
        self.tick_sink.stream()
    }

    fn buttons(&self) -> Stream<ButtonEvent> {
        self.button_sink.stream()
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
        loop {
            time = precise_time_ns();
            if time >= next_tick {
                let diff = time - next_tick;
                let delta = diff - diff % self.tick_length;
                next_tick += delta;
                for ev in self.display.poll_events() {
                    self.dispatch(ev);
                }
                self.tick_sink.send(delta);
            }
            else {
                sleep(Duration::nanoseconds(next_tick as i64 - time as i64));
            }
        }
    }
}
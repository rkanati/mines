
#![windows_subsystem = "windows"]

mod activity;
mod animator;
mod assets;
mod configure;
mod game;
mod grid;
mod play;

use {
    crate::{
        activity::{Activity, Context},
        animator::Animator,
        assets::Assets,
        configure::Configure,
        play::Play,
    },
    std::rc::Rc,
    ggez::{
        self,
        graphics,
        GameResult,
    },
    winit,
};

pub mod maths {
    use ggez::nalgebra as na;
    pub type P2 = na::Point2<f32>;
    pub type V1 = na::Vector1<f32>;
    pub type V2 = na::Vector2<f32>;
    pub type V4 = na::Vector4<f32>;
}

//use maths::*;

struct Animation {
    start:    std::time::Instant,
    rate:     f32,
    function: Rc<dyn Fn(&mut ggez::Context, f32) -> GameResult>,
}

impl Animation {
    fn new(
        start: std::time::Instant,
        duration: f32,
        function: impl Fn(&mut ggez::Context, f32) -> GameResult + 'static)
        -> Animation
    {
        Animation {
            start,
            rate: 1. / duration,
            function: Rc::new(function)
        }
    }

    fn draw(&self, ctx: &mut ggez::Context, now: std::time::Instant)
        -> GameResult<bool>
    {
        let t = (now - self.start).as_secs_f32() * self.rate;
        if t > 1. {
            Ok(false)
        }
        else {
            (self.function)(ctx, t)?;
            Ok(true)
        }
    }
}

struct Animations {
    vec: Vec<Animation>,
}

impl Animations {
    fn new() -> Animations {
        Animations { vec: Vec::new() }
    }

    fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    fn draw(&mut self, ctx: &mut ggez::Context, now: std::time::Instant) {
        self.vec.retain(|animation| animation.draw(ctx, now).unwrap());
    }
}

impl Animator for Animations {
    fn animate(
        &mut self,
        duration: f32,
        mesh:     std::rc::Rc<graphics::Mesh>,
        func:     Box<dyn Fn(f32) -> graphics::DrawParam>)
    {
        self.vec.push(Animation::new(
            std::time::Instant::now(),
            duration,
            move |ctx, t| {
                let params = func(t);
                graphics::draw(ctx, &*mesh, params)
            }
        ));
    }

    fn clear_animations(&mut self) {
        self.vec.clear();
    }
}




enum SomeActivity {
    Play(Play),
    Configure(Configure),
}

impl SomeActivity {
    fn inner<'a> (&'a self) -> &'a dyn Activity {
        match self {
            SomeActivity::Play(p)      => p,
            SomeActivity::Configure(c) => c,
        }
    }

    fn inner_mut<'a> (&'a mut self) -> &'a mut dyn Activity {
        match self {
            SomeActivity::Play(p)      => p,
            SomeActivity::Configure(c) => c,
        }
    }
}

struct App {
    ctx: ggez::Context,

    assets: Assets,

    // Render state
    animations: Animations,
    dirty: bool,

    // App state
    activity: SomeActivity,
}

impl App {
    fn new(mut ctx: ggez::Context) -> GameResult<App> {
        let assets = Assets::load(&mut ctx)?;

        let app = App {
            ctx,
            assets,

            animations: Animations::new(),
            dirty: true,

            activity: SomeActivity::Play(Play::new()),
        };

        Ok(app)
    }

    fn draw(&mut self) -> GameResult {
        let context = Context {
            ctx:      &mut self.ctx,
            assets:   &self.assets,
            animator: &mut self.animations,
        };

        self.activity.inner_mut().draw(context)?;

        let now = std::time::Instant::now();
        self.animations.draw(&mut self.ctx, now);

        graphics::present(&mut self.ctx)?;
        self.dirty = false;
        Ok(())
    }

    fn dirty(&self) -> bool {
        !self.animations.is_empty() || self.activity.inner().dirty() || self.dirty
    }

    fn handle_event(&mut self, event: ggez::event::winit_event::Event) {
        self.ctx.process_event(&event);
        use ggez::event::winit_event::{ElementState, Event::*, WindowEvent::*};
        match event {
            WindowEvent { event, .. } => match event {
                CloseRequested => ggez::event::quit(&mut self.ctx),

                MouseInput { state, button, .. } => {
                    let pos = ggez::input::mouse::position(&mut self.ctx).into();
                    let act = self.activity.inner_mut();
                    let context = Context {
                        ctx:      &mut self.ctx,
                        assets:   &self.assets,
                        animator: &mut self.animations,
                    };

                    match state {
                        ElementState::Pressed  => act.mouse_down(context, button, pos),
                        ElementState::Released => act  .mouse_up(context, button, pos)
                    }
                }

                Refresh => self.dirty = true,

                _ => { }
            },
            _ => { }
        }
    }

    fn run(&mut self, mut event_loop: ggez::event::EventsLoop) -> GameResult {
        while self.ctx.continuing {
            event_loop.poll_events(|event| self.handle_event(event));

            if !self.dirty() {
                event_loop.run_forever(|event| {
                    self.handle_event(event);
                    winit::ControlFlow::Break
                });
            }

            if self.dirty() {
                self.draw()?;
            }
        }

        Ok(())
    }
}

fn main() -> GameResult {
    let window_mode = ggez::conf::WindowMode {
        width:  play::WINDOW_WIDTH,
        height: play::WINDOW_HEIGHT,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width:  0.,
        max_width:  0.,
        min_height: 0.,
        max_height: 0.,
        resizable: false,
    };

    let window_setup = ggez::conf::WindowSetup {
        title: "Mines".to_owned(),
        samples: ggez::conf::NumSamples::Zero,
        vsync: true,
        icon: "".to_owned(),
        srgb: true,
    };

    let (ctx, event_loop) = ggez::ContextBuilder::new("mines", "rkanati.github.io")
        .add_zipfile_bytes(std::borrow::Cow::from(&include_bytes!("../resources.zip")[..]))
        .window_mode(window_mode)
        .window_setup(window_setup)
        .build()?;

    let app = &mut App::new(ctx)?;
    app.run(event_loop)
}



#![windows_subsystem = "windows"]

mod game;
mod grid;

use {
    crate::game::*,
    ggez::{
        self,
        graphics,
        input::mouse::MouseButton,
        Context, GameResult,
        nalgebra as na,
    },
    winit,
};




const GRID_WIDTH:  usize = 16;
const GRID_HEIGHT: usize = (GRID_WIDTH / 4) * 3;
const GRID_AREA:   usize = GRID_WIDTH * GRID_HEIGHT;
const N_MINES: usize = GRID_AREA / 7;




const TILE_SIZE:  f32 = 40.;

const BAR_SCALE:  f32 = 1.5;
const BAR_HEIGHT: f32 = TILE_SIZE * BAR_SCALE;



const TONE_FOREGROUND: f32 = 0.50;
const TONE_BACKGROUND: f32 = 0.07;

type P2 = na::Point2<f32>;
type V2 = na::Vector2<f32>;

struct Texts {
    flag: graphics::Text,
    mine: graphics::Text,
    tick: graphics::Text,
    nope: graphics::Text,

    okay: graphics::Text,
    woop: graphics::Text,
    ohno: graphics::Text,
    boop: graphics::Text,

    stat: graphics::Text,

    digits: Vec<graphics::Text>,
}

fn digit_color(digit: usize) -> graphics::Color {
    const H: f32 = 1.0;
    const L: f32 = 0.4;
    const DIGIT_COLORS: [[f32; 4]; 8] = [
        [L, L, H, 1.], // 1
        [L, H, L, 1.], // 2
        [H, L, L, 1.], // 3
        [H, H, L, 1.], // 4
        [H, L, H, 1.], // 5
        [L, H, H, 1.], // 6
        [H, H, H, 1.], // 7
        [H, H, H, 1.], // 8
    ];
    DIGIT_COLORS[digit - 1].into()
}

fn load_emoji(ch: char, font: graphics::Font, size: f32, color: impl Into<graphics::Color>)
    -> graphics::Text
{
    let frag = graphics::TextFragment::new(ch)
        .color(color.into())
        .font(font)
        .scale(graphics::Scale { x: size, y: size });
    let mut text = graphics::Text::new(frag);
    text.set_bounds(P2::new(size, size), graphics::Align::Center);
    text
}

impl Texts {
    fn new(ctx: &mut Context) -> GameResult<Texts> {
        //🙂🤩🤯
        let symbola = graphics::Font::new(ctx, "Symbola.ttf")?;

        //💣🏱 🏲 🏳 🗴🗸✓✔✗✘
        let noto_syms_2 = graphics::Font::new(ctx, "NotoSansSymbols2-Regular.ttf")?;

        //12345678
        let signika = graphics::Font::new(ctx, "Signika-SemiBold.ttf")?;

        let texts = Texts {
            flag: load_emoji('🏲',  noto_syms_2, TILE_SIZE, (0.4, 0.7, 1.0)),
            mine: load_emoji('💣', noto_syms_2, TILE_SIZE, (0.0, 0.0, 0.0)),
            tick: load_emoji('✓',  noto_syms_2, TILE_SIZE, (0.0, 0.0, 0.0)),
            nope: load_emoji('✗',  noto_syms_2, TILE_SIZE, (1.0, 1.0, 1.0)),

            okay: load_emoji('🙂', symbola, BAR_HEIGHT, (0.6, 0.6, 0.6)),
            woop: load_emoji('🤩', symbola, BAR_HEIGHT, (1.0, 0.4, 0.7)),
            ohno: load_emoji('🤯', symbola, BAR_HEIGHT, (0.8, 0.3, 0.0)),
            boop: load_emoji('😲', symbola, BAR_HEIGHT, (0.3, 0.6, 1.0)),

            stat: {
                let mut text = load_emoji('🏲', noto_syms_2, BAR_HEIGHT, (1., 1., 1.));
                text.add(("", signika, BAR_HEIGHT));
                text
            },

            digits: (1 ..= 8)
                .map(|digit| {
                    let ch = std::char::from_digit(digit as u32, 10).unwrap();
                    load_emoji(ch, signika, TILE_SIZE, digit_color(digit))
                })
                .collect(),
        };

        Ok(texts)
    }
}

struct App {
    state: game::State,
    boop: bool,

    square: graphics::Mesh,
    texts: Texts,
}

impl App {
    fn new(ctx: &mut Context, state: game::State) -> GameResult<App> {
        let square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(1., 1., TILE_SIZE - 2., TILE_SIZE - 2.),
            (1., 1., 1.).into()
        )?;

        let texts = Texts::new(ctx)?;

        let app = App {
            state,
            boop: false,

            square,
            texts,
        };

        Ok(app)
    }

    fn mouse_down(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if y < BAR_HEIGHT {
            let bar_rect = graphics::screen_coordinates(ctx);
            if (x - bar_rect.w * 0.5).abs() < BAR_HEIGHT * 0.5 {
                self.boop = true;
                self.state.restart();
            }
        }
        else {
            let i = (( x               / TILE_SIZE).trunc() as i32).min(GRID_WIDTH  as i32).max(0);
            let j = (((y - BAR_HEIGHT) / TILE_SIZE).trunc() as i32).min(GRID_HEIGHT as i32).max(0);
            let ij = grid::Coords::new(i, j);

            match button {
                MouseButton::Left  => self.state .dig(ij),
                MouseButton::Right => self.state.flag(ij),
                _ => { return; }
            };
        }

        self.draw(ctx).unwrap();
    }

    fn mouse_up(&mut self, ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        if self.boop {
            self.boop = false;
            self.draw(ctx).unwrap();
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        use graphics::{clear, draw, DrawParam};

      //const COLOR_BACKGROUND: [f32; 4] = [TONE_BACKGROUND; 4];
        clear(ctx, (0., 0., 0.).into());

        let mut bar_rect = graphics::screen_coordinates(ctx);
        bar_rect.h = BAR_HEIGHT;

      //let bar = graphics::Mesh::new_rectangle(
      //    ctx,
      //    DrawMode::fill(),
      //    bar_rect,
      //    COLOR_BACKGROUND.into()
      //)?;

        bar_rect.w -= 6.;
        bar_rect.x += 3.;

      //let params = DrawParam::new();
      //draw(ctx, &bar, params)?;

        let face = if self.boop {
            &self.texts.boop
        }
        else {
            use game::Status::*;
            match self.state.status() {
                Playing => &self.texts.okay,
                Won     => &self.texts.woop,
                Dead    => &self.texts.ohno
            }
        };

        let params = DrawParam::new()
            .dest(P2::new((bar_rect.w - BAR_HEIGHT) * 0.5, 0.));
        draw(ctx, face, params)?;

        let mut count = self.texts.stat.clone();
        count.fragments_mut()[1].text = format!("{:3}", self.state.flags_remaining());
        count.set_bounds(P2::new(bar_rect.w - 6., bar_rect.h), graphics::Align::Right);
        draw(ctx, &count, DrawParam::new().dest(P2::new(3., 0.)))?;

        for (coords, tile) in self.state.enumerate_tiles() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::Hasher;
            hasher.write_i32(coords.x);
            hasher.write_i32(coords.y);
            let frac = hasher.finish() as f64 / std::u64::MAX as f64;

            let sh = 0.6 + 0.4 * frac as f32;
            let c = sh * TONE_FOREGROUND;

            let position = P2::new(0., BAR_HEIGHT)
                         + V2::new(coords.x as f32, coords.y as f32) * TILE_SIZE;
            let params = DrawParam::new()
                .dest(position);

            match tile.state {
                TileState::Covered(flag) => {
                    if self.state.done() {
                        if flag {
                            match tile.kind {
                                TileKind::Mine => {
                                    draw(ctx, &self.square, params.color((0., c, 0.).into()))?;
                                    draw(ctx, &self.texts.tick, params)?;
                                }

                                TileKind::Dirt => {
                                    draw(ctx, &self.square, params.color((c, 0., 0.).into()))?;
                                    draw(ctx, &self.texts.nope, params)?;
                                }
                            }
                        }
                        else {
                            match tile.kind {
                                TileKind::Mine => {
                                    draw(ctx, &self.square, params.color((c, 0., 0.).into()))?;
                                    draw(ctx, &self.texts.mine, params)?;
                                }

                                TileKind::Dirt => {
                                    draw(ctx, &self.square, params.color((c, c, c).into()))?;
                                }
                            }
                        }
                    }
                    else {
                        if flag {
                            draw(ctx, &self.square, params.color((0., c, c).into()))?;
                            draw(ctx, &self.texts.flag, params)?;
                        }
                        else {
                            draw(ctx, &self.square, params.color((c, c, c).into()))?;
                        }
                    }
                }

                TileState::Uncovered => {
                    match tile.kind {
                        TileKind::Dirt => {
                            let c = sh * TONE_BACKGROUND;
                            draw(ctx, &self.square, params.color((c, c, c).into()))?;

                            if tile.n_near > 0 {
                                let i = tile.n_near - 1;
                                let text = &self.texts.digits[i];
                                draw(ctx, text, params)?;
                            }
                        }

                        TileKind::Mine => {
                            draw(ctx, &self.square, params.color((1., 1., 0.).into()))?;
                            draw(ctx, &self.texts.mine, params)?;
                        }
                    }
                }
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let width  = GRID_WIDTH  as f32 * TILE_SIZE;
    let height = (GRID_HEIGHT as f32 * TILE_SIZE) + BAR_HEIGHT;

    let window_mode = ggez::conf::WindowMode {
        width,
        height,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width:  width,
        max_width:  width,
        min_height: height,
        max_height: height,
        resizable: false,
    };

    let window_setup = ggez::conf::WindowSetup {
        title: "Mines".to_owned(),
        samples: ggez::conf::NumSamples::Zero,
        vsync: true,
        icon: "".to_owned(),
        srgb: true,
    };

    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("mines", "rkanati.github.io")
        .add_zipfile_bytes(std::borrow::Cow::from(&include_bytes!("../resources.zip")[..]))
        .window_mode(window_mode)
        .window_setup(window_setup)
        .build()?;

    let config = game::Config {
        width:   GRID_WIDTH,
        height:  GRID_HEIGHT,
        n_mines: N_MINES,
        seed:    None
    };
    let state = game::State::new(config);

    let app = &mut App::new(ctx, state)?;

    event_loop.run_forever(|event| {
        ctx.process_event(&event);
        use ggez::event::winit_event::{ElementState, Event::*, WindowEvent::*};
        match event {
            WindowEvent { event, .. } => match event {
                CloseRequested => return winit::ControlFlow::Break,

                MouseInput { state, button, .. } => {
                    let pos = ggez::input::mouse::position(ctx);
                    match state {
                        ElementState::Pressed  => app.mouse_down(ctx, button, pos.x, pos.y),
                        ElementState::Released => app  .mouse_up(ctx, button, pos.x, pos.y)
                    }
                }

                Refresh => app.draw(ctx).unwrap(),

                _ => { }
            },
            _ => { }
        }

        winit::ControlFlow::Continue
    });

    Ok(())
}


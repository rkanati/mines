
use {
    super::maths::*,
    crate::{
        activity::*,
        game,
        animator::SimpleKey,
    },
    ggez::{
        GameResult,
        input::mouse::MouseButton,
    },
};

const GRID_WIDTH:  usize = 12;
const GRID_HEIGHT: usize = (GRID_WIDTH / 4) * 3;
const GRID_AREA:   usize = GRID_WIDTH * GRID_HEIGHT;
const N_MINES:     usize = GRID_AREA / 8;

// TODO un-pub these
pub const TILE_SIZE:  f32 = 30.;

const BAR_SCALE:  f32 = 1.5;
pub const BAR_HEIGHT: f32 = TILE_SIZE * BAR_SCALE;

const TONE_FOREGROUND: f32 = 0.50;
const TONE_BACKGROUND: f32 = 0.07;

pub const WINDOW_WIDTH:  f32 = GRID_WIDTH  as f32 * TILE_SIZE;
pub const WINDOW_HEIGHT: f32 = GRID_HEIGHT as f32 * TILE_SIZE + BAR_HEIGHT;

pub struct Play {
    state: game::State,
    boop:  bool,
    dirty: bool,
}

impl Play {
    pub fn new() -> Play {
        let config = game::Config {
            width:   GRID_WIDTH,
            height:  GRID_HEIGHT,
            n_mines: N_MINES,
            seed:    None
        };

        let state = game::State::new(config);

        Play { state, boop: false, dirty: true }
    }
}

impl Activity for Play {
    fn mouse_down<'a> (
        &mut self,
        Context { ctx, assets, animator }: Context<'a>,
        button: MouseButton, position: P2)
    {
        if position.y < BAR_HEIGHT {
            let bar_rect = ggez::graphics::screen_coordinates(ctx);
            if (position.x - bar_rect.w * 0.5).abs() < BAR_HEIGHT * 0.5 {
                self.boop = true;
                self.state.restart();
                animator.clear_animations();
            }
        }
        else {
            let ij = (position.coords - V2::new(0., BAR_HEIGHT))
                .map(|x| ((x / TILE_SIZE).trunc() as i32))
                .zip_map(&self.state.dims(), i32::min)
                .map(|x| x.max(0))
                .into();

            match button {
                MouseButton::Left => {
                    let dug = self.state.dig(ij);
                    for (ij, boom) in dug.iter().copied() {
                        let boom = match boom {
                            Some(boom) => boom,
                            None       => continue
                        };

                        let center = P2::new(0., BAR_HEIGHT)
                                   + ij.coords.map(|x| (x as f32 + 0.5) * TILE_SIZE);

                        let key = SimpleKey::new()
                            .position(center);

                        if boom {
                            animator.animate_simple(
                                2.0,
                                assets.circle.clone(),
                                key.color(V4::new(1.0, 1.0, 0.0, 0.5)),
                                key.color(V4::new(0.3, 0.0, 0.0, 0.0))
                                    .scale(V2::repeat(TILE_SIZE * 30.)),
                            );
                        }
                        else {
                            animator.animate_simple(
                                0.2,
                                assets.circle.clone(),
                                key.color(V4::new(1.0, 1.0, 1.0, 0.5)),
                                key.color(V4::new(0.0, 0.5, 1.0, 0.0))
                                    .scale(V2::repeat(TILE_SIZE * 1.2)),
                            );
                        }
                    }

                }

                MouseButton::Right => {
                    self.state.flag(ij);
                }

                _ => { }
            }

            if self.state.status() == game::Status::Won {
                let bar_rect = ggez::graphics::screen_coordinates(ctx);
                let bar_center = P2::new(bar_rect.w * 0.5, BAR_HEIGHT * 0.5);

                let key = SimpleKey::new()
                    .position(bar_center);

                animator.animate_simple(
                    1.5,
                    assets.star.clone(),
                    key.color(V4::new(1.0, 1.0, 1.0, 1.)),
                    key.color(V4::new(1.0, 0.4, 0.7, 0.))
                        .scale(V2::repeat(bar_rect.w * 1.5))
                        .angle(10.),
                );
            }
        }

        self.dirty = true;
    }

    fn mouse_up<'a> (&mut self, _context: Context<'a>, button: MouseButton, position: P2) {
        if self.boop {
            self.boop = false;
            self.dirty = true;
        }
    }

    fn draw<'a> (
        &mut self,
        Context { ctx, assets, animator }: Context<'a>)
        -> GameResult
    {
        use ggez::graphics::{Align, clear, draw, DrawParam, screen_coordinates};

        let now = std::time::Instant::now();

        clear(ctx, (0., 0., 0.).into());

        let bar_rect = {
            let mut rect = screen_coordinates(ctx);
            rect.h = BAR_HEIGHT;
            rect.w -= 6.;
            rect.x += 3.;
            rect
        };

        let face = if self.boop {
            &assets.texts.boop
        }
        else {
            use game::Status::*;
            match self.state.status() {
                Playing => &assets.texts.okay,
                Won     => &assets.texts.woop,
                Dead    => &assets.texts.ohno
            }
        };

        let bar_center = (bar_rect.w - BAR_HEIGHT) * 0.5;

        let params = DrawParam::new()
            .dest(P2::new(bar_center, 0.));
        draw(ctx, face, params)?;

        let mut count = assets.texts.stat.clone();
        count.fragments_mut()[1].text = format!("{:3}", self.state.flags_remaining());
        count.set_bounds(P2::new(bar_rect.w - 6., bar_rect.h), Align::Right);
        draw(ctx, &count, DrawParam::new().dest(P2::new(3., 0.)))?;

        for (coords, tile) in self.state.enumerate_tiles() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::Hasher;
            hasher.write_i32(coords.x);
            hasher.write_i32(coords.y);
            let hash = hasher.finish();

            let hazard = &assets.texts.hazards[hash as usize % assets.texts.hazards.len()];

            let frac = hash as f64 / std::u64::MAX as f64;

            let sh = 0.6 + 0.4 * frac as f32;
            let c = sh * TONE_FOREGROUND;

            let position = P2::new(0., BAR_HEIGHT)
                         + V2::new(coords.x as f32, coords.y as f32) * TILE_SIZE;
            let params = DrawParam::new()
                .dest(position);

            use game::{TileState, TileKind};
            match tile.state {
                TileState::Covered(flag) => {
                    if self.state.done() {
                        if flag {
                            match tile.kind {
                                TileKind::Mine => {
                                    draw(ctx, &assets.square, params.color((0., c, 0.).into()))?;
                                    draw(ctx, &assets.texts.tick, params)?;
                                }

                                TileKind::Dirt => {
                                    draw(ctx, &assets.square, params.color((c, 0., 0.).into()))?;
                                    draw(ctx, &assets.texts.nope, params)?;
                                }
                            }
                        }
                        else {
                            match tile.kind {
                                TileKind::Mine => {
                                    draw(ctx, &assets.square, params.color((c, 0., 0.).into()))?;
                                    draw(ctx, hazard, params)?;
                                }

                                TileKind::Dirt => {
                                    draw(ctx, &assets.square, params.color((c, c, c).into()))?;
                                }
                            }
                        }
                    }
                    else {
                        if flag {
                            draw(ctx, &assets.square, params.color((0., c, c).into()))?;
                            draw(ctx, &assets.texts.flag, params)?;
                        }
                        else {
                            draw(ctx, &assets.square, params.color((c, c, c).into()))?;
                        }
                    }
                }

                TileState::Uncovered => {
                    match tile.kind {
                        TileKind::Dirt => {
                            let c = sh * TONE_BACKGROUND;
                            draw(ctx, &assets.square, params.color((c, c, c).into()))?;

                            if tile.n_near > 0 {
                                let i = tile.n_near - 1;
                                let text = &assets.texts.digits[i];
                                draw(ctx, text, params)?;
                            }
                        }

                        TileKind::Mine => {
                            draw(ctx, &assets.square, params.color((1., 1., 0.).into()))?;
                            let mut hazard = hazard.clone();
                            hazard.fragments_mut()[0].color = None;//Some(graphics::BLACK;
                            draw(ctx, &hazard, params.color(ggez::graphics::BLACK))?;
                        }
                    }
                }
            }
        }

        self.dirty = false;
        Ok(())
    }

    fn dirty(&self) -> bool {
        self.dirty
    }
}


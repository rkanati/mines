
use {
    crate::grid::{Coords, Grid},
    std::collections::VecDeque,
    ggez::nalgebra::Vector2 as V2,
    rand::{distributions::Uniform, Rng, SeedableRng},
    rand_pcg::Pcg32,
};

fn flood_clear(grid: &mut Grid<Tile>, start: Coords) {
    let w = grid.width() as i32;
    let h = grid.height() as i32;

    let mut q = VecDeque::new();
    q.push_back(start);

    let mut do_adj = |q: &mut VecDeque<Coords>, at: Coords| {
        if at.x < 0 || at.y < 0 || at.x >= w || at.y >= h {
            return;
        }

        let tile = &mut grid[at];
        if tile.kind == TileKind::Dirt && tile.state != TileState::Uncovered {
            tile.state = TileState::Uncovered;
            if tile.n_near == 0 { q.push_back(at); }
        }
    };

    while let Some(p) = q.pop_front() {
        do_adj(&mut q, p + V2::new(-1,  0));
        do_adj(&mut q, p + V2::new( 0, -1));
        do_adj(&mut q, p + V2::new( 1,  0));
        do_adj(&mut q, p + V2::new( 0,  1));
        do_adj(&mut q, p + V2::new(-1, -1));
        do_adj(&mut q, p + V2::new( 1, -1));
        do_adj(&mut q, p + V2::new(-1,  1));
        do_adj(&mut q, p + V2::new( 1,  1));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Won,
    Dead
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    Dirt,
    Mine
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileState {
    Covered(bool),
    Uncovered
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub kind:    TileKind,
    pub state:   TileState,
    pub n_near:  usize,
}

impl Tile {
    fn new() -> Tile {
        Tile {
            kind:   TileKind::Dirt,
            state:  TileState::Covered(false),
            n_near: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub width:   usize,
    pub height:  usize,
    pub n_mines: usize,
    pub seed:    Option<u64>,
}

pub struct State {
    config:  Config,
    tiles:   Grid<Tile>,
    status:  Status,
    n_flags: usize,
}

impl State {
    pub fn new(config: Config) -> State {
        let mut tiles: Grid<Tile> = Grid::new_fill(config.width, config.height, Tile::new());

        let seed = config.seed.unwrap_or(rand::rngs::OsRng.gen());
        let mut rng = Pcg32::seed_from_u64(seed);
        let i_distro = Uniform::new(0, config.width  as i32);
        let j_distro = Uniform::new(0, config.height as i32);

        // randomly position mines
        for _ in 0 .. config.n_mines {
            let ij = loop {
                let i = rng.sample(&i_distro);
                let j = rng.sample(&j_distro);
                let ij = Coords::new(i, j);
                match tiles[ij].kind {
                    TileKind::Dirt => break ij,
                    TileKind::Mine => continue
                }
            };
            tiles[ij].kind = TileKind::Mine;
        }

        // compute nearby-mine counts
        for j in 0 .. config.height as i32 {
            for i in 0 .. config.width as i32 {
                let ij = Coords::new(i, j);
                for dj in -1 ..= 1 {
                    for di in -1 ..= 1 {
                        let adj = ij + V2::new(di, dj);
                        if tiles.in_bounds(adj) && tiles[adj].kind == TileKind::Mine {
                            tiles[ij].n_near += 1;
                        }
                    }
                }
            }
        }

        // make the first dig automatically
        let start_ij = loop {
            let i = rng.sample(&i_distro);
            let j = rng.sample(&j_distro);
            let ij = Coords::new(i, j);
            if tiles[ij].n_near == 0 {
                break ij;
            }
        };

        let mut state = State {
            config,
            tiles,
            status: Status::Playing,
            n_flags: config.n_mines,
        };

        state.dig(start_ij);

        state
    }

    fn uncover(&mut self, ij: Coords) {
        let tile = &mut self.tiles[ij];
        if tile.state == TileState::Covered(true) {
            return;
        }

        tile.state = TileState::Uncovered;

        if tile.kind == TileKind::Mine {
            self.status = Status::Dead;
        }
        else if tile.n_near == 0 {
            flood_clear(&mut self.tiles, ij);
        }
    }

    pub fn dig(&mut self, ij: Coords) {
        if self.done() { return; }

        let tile = &mut self.tiles[ij];
        match tile.state {
            TileState::Covered(false) => {
                self.uncover(ij);
            }

            TileState::Uncovered => {
                for dj in -1 ..= 1 {
                    for di in -1 ..= 1 {
                        let adj = ij + V2::new(di, dj);
                        if self.tiles.in_bounds(adj) {
                            self.uncover(adj);
                        }
                    }
                }
            }

            _ => { }
        }

        self.check_win();
    }

    pub fn flag(&mut self, ij: Coords) {
        if self.done() { return; }

        if let TileState::Covered(flag) = &mut self.tiles[ij].state {
            if !*flag && self.n_flags != 0 {
                *flag = true;
                self.n_flags -= 1
            }
            else {
                *flag = false;
                self.n_flags += 1;
            }
        }

        self.check_win();
    }

    fn check_win(&mut self) {
        if !self.done() {
            if self.tiles.iter()
                .filter(|tile| match tile.kind {
                    TileKind::Dirt => tile.state != TileState::Uncovered,
                    TileKind::Mine => tile.state != TileState::Covered(true),
                })
                .count() == 0
            {
                self.status = Status::Won;
            }
        }
    }

    pub fn enumerate_tiles<'a> (&'a self) -> impl Iterator<Item = (Coords, &'a Tile)> + 'a {
        self.tiles.enumerate()
    }

    pub fn done(&self) -> bool {
        self.status != Status::Playing
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn restart(&mut self) {
        let config = Config { seed: None, ..self.config };
        *self = Self::new(config);
    }

    pub fn flags_remaining(&self) -> usize {
        self.n_flags
    }
}


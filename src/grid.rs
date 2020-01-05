
use {
    ggez::nalgebra as na,
};

pub type Coords = na::Point2<i32>;

pub struct Grid<T> {
    vec:    Vec<T>,
    width:  usize,
    height: usize,
}

impl<T> Grid<T> where T: Clone + Default {
    #[allow(unused)]
    pub fn new(width: usize, height: usize) -> Grid<T> {
        Self::new_fill(width, height, <T as Default>::default())
    }
}

impl<T> Grid<T> where T: Clone {
    pub fn new_fill(width: usize, height: usize, with: impl Into<T>) -> Grid<T> {
        let elem = with.into();
        Self::new_generate(width, height, move |_| elem.clone())
    }

    pub fn new_generate(width: usize, height: usize, func: impl Fn(Coords) -> T) -> Grid<T> {
        let vec = (0 .. height as i32)
            .flat_map(|y| {
                let func = &func;
                (0 .. width as i32)
                    .map(move |x| func(Coords::new(x, y)))
            })
            .collect();

        Grid { vec, width, height }
    }

    pub fn indices<'a> (&'a self) -> impl Iterator<Item = Coords> + 'a {
        (0 .. self.height as i32)
            .flat_map(move |y|
                (0 .. self.width as i32)
                    .map(move |x| Coords::new(x, y))
            )
    }

    pub fn enumerate<'a> (&'a self) -> impl Iterator<Item = (Coords, &'a T)> + 'a {
        self.indices().map(move |ij| (ij, &self[ij]))
    }

    pub fn iter<'a> (&'a self) -> impl Iterator<Item = &'a T> + 'a {
        self.vec.iter()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn in_bounds(&self, p: Coords) -> bool {
           p.x >= 0 && p.x < self.width  as i32
        && p.y >= 0 && p.y < self.height as i32
    }
}

impl<T> std::ops::Index<Coords> for Grid<T> {
    type Output = T;
    fn index(&self, p: Coords) -> &T {
        self.vec.get(self.width * p.y as usize + p.x as usize).unwrap()
    }
}

impl<T> std::ops::IndexMut<Coords> for Grid<T> {
    fn index_mut(&mut self, p: Coords) -> &mut T {
        self.vec.get_mut(self.width * p.y as usize + p.x as usize).unwrap()
    }
}


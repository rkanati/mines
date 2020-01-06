
use {
    crate::{
        animator::Animator,
        assets::Assets,
        maths::*,
    },
    ggez::{
        GameResult,
        input::mouse::MouseButton,
    },
};

pub struct Context<'a> {
    pub ctx:      &'a mut ggez::Context,
    pub assets:   &'a Assets,
    pub animator: &'a mut dyn Animator,
}

pub trait Activity {
    fn mouse_down<'a> (&mut self, context: Context<'a>, button: MouseButton, position: P2);
    fn mouse_up  <'a> (&mut self, context: Context<'a>, button: MouseButton, position: P2);
    fn draw      <'a> (&mut self, context: Context<'a>) -> GameResult;
    fn dirty(&self) -> bool;
}



use {
    crate::{
        activity::*,
        maths::*,
    },
    ggez::{
        GameResult,
        input::mouse::MouseButton,
    },
};

pub struct Configure {
}

impl Configure {
}

impl Activity for Configure {
    fn mouse_down<'a> (&mut self, context: Context<'a>, button: MouseButton, position: P2) {
    }

    fn mouse_up<'a> (&mut self, context: Context<'a>, button: MouseButton, position: P2) {
    }

    fn draw<'a> (&mut self, context: Context<'a>) -> GameResult {
        Ok(())
    }

    fn dirty(&self) -> bool {
        false
    }
}


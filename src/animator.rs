
pub use {
    crate::{
        assets::Assets,
        maths::*,
    },
    ggez::{
        Context,
        GameResult,
        graphics,
        input::mouse::MouseButton,
    },
};

#[derive(Clone, Copy)]
pub struct SimpleKey {
    pub position: P2,
    pub scale:    V2,
    pub angle:    V1,
    pub color:    V4,
}

impl SimpleKey {
    pub fn new() -> SimpleKey {
        SimpleKey {
            position: V2::zeros().into(),
            scale:    V2::zeros(),
            angle:    V1::zeros(),
            color:    V4::zeros(),
        }
    }

    pub fn position(self, position: P2) -> Self {
        SimpleKey { position, ..self }
    }

    pub fn scale(self, scale: V2) -> Self {
        SimpleKey { scale, ..self }
    }

    pub fn angle(self, angle: f32) -> Self {
        SimpleKey { angle: V1::new(angle), ..self }
    }

    pub fn color(self, color: V4) -> Self {
        SimpleKey { color, ..self }
    }
}

pub trait Animator {
    fn animate(
        &mut self,
        duration: f32,
        mesh:     std::rc::Rc<graphics::Mesh>,
        func:     Box<dyn Fn(f32) -> graphics::DrawParam>,
    );

    fn animate_simple(
        &mut self,
        duration: f32,
        mesh:     std::rc::Rc<graphics::Mesh>,
        start:    SimpleKey,
        end:      SimpleKey)
    {
        self.animate(
            duration,
            mesh,
            Box::new(move |t| {
                let position = start.position.coords.lerp(&end.position.coords, t);
                let scale = start.scale.lerp(&end.scale, t);
                let angle = start.angle.lerp(&end.angle, t).x;
                let color: [f32; 4] = start.color.lerp(&end.color, t).into();
                graphics::DrawParam::new()
                    .dest(P2::from(position))
                    .scale(scale)
                    .rotation(angle)
                    .color(color.into())
            })
        )
    }

    fn clear_animations(&mut self);
}


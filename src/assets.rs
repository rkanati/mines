
use {
    crate::{
        maths::*,
        play::*, // TODO remove
    },
    ggez::{
        Context,
        GameResult,
        graphics::*,
    },
};

pub struct Texts {
    pub flag: Text,
    pub tick: Text,
    pub nope: Text,

    pub okay: Text,
    pub woop: Text,
    pub ohno: Text,
    pub boop: Text,

    pub hazards: Vec<Text>,

    pub stat: Text,

    pub digits: Vec<Text>,
}

fn digit_color(digit: usize) -> Color {
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

fn load_emoji(ch: char, font: Font, size: f32, color: impl Into<Color>)
    -> Text
{
    let frag = TextFragment::new(ch)
        .color(color.into())
        .font(font)
        .scale(Scale { x: size, y: size });
    let mut text = Text::new(frag);
    text.set_bounds(P2::new(size, size), Align::Center);
    text
}

impl Texts {
    fn new(ctx: &mut Context) -> GameResult<Texts> {
        let symbola = Font::new(ctx, "Symbola.ttf")?;
        let signika = Font::new(ctx, "Signika-SemiBold.ttf")?;

        //🦟
        //🧨
        //🧪
        const HAZARDS: &'static str = "💀☢☣⚡💣🦇🦈🦀💩🦂🦑🤖🕷🦖🕱";

        let texts = Texts {
            flag: load_emoji('⚑',  symbola, TILE_SIZE, (0.4, 0.7, 1.0)),
            tick: load_emoji('✓',  symbola, TILE_SIZE, (0.0, 0.0, 0.0)),
            nope: load_emoji('✗',  symbola, TILE_SIZE, (1.0, 1.0, 1.0)),

            okay: load_emoji('🙂', symbola, BAR_HEIGHT, (0.6, 0.6, 0.6)),
            woop: load_emoji('🤩', symbola, BAR_HEIGHT, (1.0, 0.4, 0.7)),
            ohno: load_emoji('🤯', symbola, BAR_HEIGHT, (0.8, 0.3, 0.0)),
            boop: load_emoji('😲', symbola, BAR_HEIGHT, (0.3, 0.6, 1.0)),

            hazards: HAZARDS.chars()
                .map(|ch| load_emoji(ch, symbola, TILE_SIZE, (1., 1., 1.)))
                .collect(),

            stat: {
                let mut text = load_emoji('⚑', symbola, BAR_HEIGHT, (1., 1., 1.));
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

pub struct Assets {
    pub square: Mesh,
    pub circle: std::rc::Rc<Mesh>,
    pub star:   std::rc::Rc<Mesh>,
    pub texts:  Texts,
}

impl Assets {
    pub fn load(ctx: &mut ggez::Context) -> GameResult<Assets> {
        let square = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(1., 1., TILE_SIZE - 2., TILE_SIZE - 2.),
            (1., 1., 1.).into()
        )?;

        let circle = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            P2::new(0., 0.),
            1.,
            0.005,
            (1., 1., 1.).into()
        )?;

        let star_points: Vec<P2> = (0..10)
            .map(|i| {
                let theta = i as f32 * std::f32::consts::PI * 0.2;
                let dist = 0.5 * (1. + (i % 2) as f32);
                P2::new(dist * theta.cos(), dist * theta.sin())
            })
            .collect();

        let star = Mesh::new_polygon(
            ctx,
            DrawMode::fill(),
            &star_points,
            (1., 1., 1.).into()
        )?;

        let texts = Texts::new(ctx)?;

        let assets = Assets {
            square,
            circle: std::rc::Rc::new(circle),
            star:   std::rc::Rc::new(star),
            texts,
        };

        Ok(assets)
    }
}


const VC07_VERSION: &str = "VC0.8";
const START_POWER_LEVEL: u8 = 1;
const MAX_POWER_LEVEL: u8 = 5;
const POWERUP_SOUND: SoundId = SoundId::new("void_canticle.powerup");
const POWER_RELIC: Pixel = Pixel::rgb(184, 110, 235);
const POWER_RELIC_LIGHT: Pixel = Pixel::rgb(255, 220, 150);
const PILGRIM_VIOLET: Pixel = Pixel::rgb(154, 92, 224);
const PILGRIM_GOLD: Pixel = Pixel::rgb(216, 164, 72);
const PILGRIM_SHADOW: Pixel = Pixel::rgb(38, 42, 58);
const PILGRIM_THRUSTER: Pixel = Pixel::rgb(184, 102, 255);

#[derive(Debug, Clone, Copy)]
struct PowerShot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    damage: u32,
    radius: u32,
    alive: bool,
}

#[derive(Debug, Clone, Copy)]
struct RelicDrop {
    x: f32,
    y: f32,
    age: f32,
    phase: f32,
    alive: bool,
}

#[derive(Debug, Clone, Copy)]
struct WeaponProfile {
    period: f32,
    radius: u32,
    volley_damage: u32,
    name: &'static str,
}

struct PilgrimV07Visuals {
    idle: Vec<Sprite>,
    focus: Vec<Sprite>,
}

impl PilgrimV07Visuals {
    fn new() -> Self {
        let palette = [
            ('P', PILGRIM),
            ('D', PILGRIM_DARK),
            ('S', PILGRIM_SHADOW),
            ('V', PILGRIM_VIOLET),
            ('G', PILGRIM_GOLD),
            ('F', PILGRIM_THRUSTER),
        ];

        let idle_a = sprite_from_ascii(
            &[
                "........GGG........",
                ".......GPPPG.......",
                "......GPPDPPG......",
                ".....GGDDDDDGG.....",
                "...GGPPDDDDDPPGG...",
                "..GPPDDDVVDDDPPG...",
                ".GPPDDDVVVDDDPPG...",
                ".GPD.DDVVVDD.DPG...",
                "..PD..DVVVD..DP....",
                "..PD..DGGGD..DP....",
                "...D...DGD...D.....",
                "...D..GGGGG..D.....",
                "...D...D.D...D.....",
                ".......D.D.........",
                "......DD.DD........",
                "......D...D........",
                ".....DD...DD.......",
                ".....DP...PD.......",
                ".....D.....D.......",
                ".....F.....F.......",
                "....FFF...FFF......",
                ".....F.....F.......",
                "...................",
            ],
            &palette,
        );

        let idle_b = sprite_from_ascii(
            &[
                "........GGG........",
                ".......GPPPG.......",
                "......GPPDPPG......",
                ".....GGDDDDDGG.....",
                "...GGPPDDDDDPPGG...",
                "..GPPDDDVVDDDPPG...",
                ".GPPDDDVVVDDDPPG...",
                ".GPD.DDVVVDD.DPG...",
                "..PD..DVVVD..DP....",
                "..PD..DGGGD..DP....",
                "...D...DGD...D.....",
                "...D..GGGGG..D.....",
                "...D...D.D...D.....",
                ".......D.D.........",
                "......DD.DD........",
                "......D...D........",
                ".....DD...DD.......",
                ".....DP...PD.......",
                ".....D.....D.......",
                "....FF.....FF......",
                "...FFFF...FFFF.....",
                "....FF.....FF......",
                "...................",
            ],
            &palette,
        );

        let focus_a = sprite_from_ascii(
            &[
                "........GGG........",
                ".......GPPPG.......",
                "......GPPDPPG......",
                "......GDDDDDG......",
                ".....GPPDDDPPG.....",
                ".....PDDVVVDDP.....",
                ".....PDDVVVDDP.....",
                ".....PDDVVVDDP.....",
                "......DDVDD........",
                "......DGGGD........",
                "......DGDGD........",
                "......GGGGG........",
                ".......D.D.........",
                ".......D.D.........",
                "......DD.DD........",
                "......D...D........",
                "......D...D........",
                "......P...P........",
                "......D...D........",
                "......F...F........",
                ".....FFF.FFF.......",
                "......F...F........",
                "...................",
            ],
            &palette,
        );

        let focus_b = sprite_from_ascii(
            &[
                "........GGG........",
                ".......GPPPG.......",
                "......GPPDPPG......",
                "......GDDDDDG......",
                ".....GPPDDDPPG.....",
                ".....PDDVVVDDP.....",
                ".....PDDVVVDDP.....",
                ".....PDDVVVDDP.....",
                "......DDVDD........",
                "......DGGGD........",
                "......DGDGD........",
                "......GGGGG........",
                ".......D.D.........",
                ".......D.D.........",
                "......DD.DD........",
                "......D...D........",
                "......D...D........",
                "......P...P........",
                "......D...D........",
                ".....FF...FF.......",
                "....FFFF.FFFF......",
                ".....FF...FF.......",
                "...................",
            ],
            &palette,
        );

        Self {
            idle: vec![idle_a, idle_b],
            focus: vec![focus_a, focus_b],
        }
    }

    fn render(
        &self,
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        focused: bool,
        invulnerability: f32,
        animation_time: f32,
    ) {
        let visible = invulnerability <= 0.0 || ((invulnerability * 16.0) as i32 % 2 == 0);
        if !visible {
            return;
        }

        let frames = if focused { &self.focus } else { &self.idle };
        let frame = animation_frame(animation_time, if focused { 6.0 } else { 8.0 }, frames.len());
        frames[frame].draw_centered(framebuffer, x, y);

        if focused {
            framebuffer.draw_circle(x, y - 1, 6, FOCUS_COLOR);
            framebuffer.draw_circle(x, y - 1, 9, PILGRIM_VIOLET);
            framebuffer.fill_circle(x, y - 1, 1, FOCUS_COLOR);
        }
    }
}

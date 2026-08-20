const VC13_VERSION: &str = "VC1.3";
const XP_ORB_CARRION: u32 = 6;
const XP_ORB_SPECIAL: u32 = 10;
const XP_ORB_THREAT: u32 = 12;
const BASE_MAGNET_RADIUS: f32 = 28.0;
const MAGNET_RADIUS_PER_STACK: f32 = 22.0;
const RAPID_FIRE_PER_STACK: f32 = 0.18;
const XP_GAIN_PER_STACK: u32 = 25;

const VC_LEVEL_UP: ActionId = ActionId::new("void_canticle.level.up");
const VC_LEVEL_DOWN: ActionId = ActionId::new("void_canticle.level.down");
const VC_LEVEL_CONFIRM: ActionId = ActionId::new("void_canticle.level.confirm");

const XP_ORB: Pixel = Pixel::rgb(116, 190, 255);
const XP_ORB_CORE: Pixel = Pixel::rgb(220, 242, 255);
const XP_BAR_BG: Pixel = Pixel::rgb(24, 31, 48);
const XP_BAR_FILL: Pixel = Pixel::rgb(92, 165, 238);

#[derive(Debug, Clone, Copy)]
struct XpOrb {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    age: f32,
    value: u32,
    alive: bool,
}

impl XpOrb {
    fn new(x: f32, y: f32, value: u32) -> Self {
        Self {
            x,
            y,
            vx: ((x * 0.71 + y * 0.37).sin()) * 14.0,
            vy: 8.0,
            age: 0.0,
            value,
            alive: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpgradeKind {
    RapidFire,
    MagnetField,
    StellarPower,
    XpHunger,
    VitalSpark,
    CoreSurge,
}

const UPGRADE_POOL: [UpgradeKind; 6] = [
    UpgradeKind::RapidFire,
    UpgradeKind::MagnetField,
    UpgradeKind::StellarPower,
    UpgradeKind::XpHunger,
    UpgradeKind::VitalSpark,
    UpgradeKind::CoreSurge,
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct BuildState {
    rapid_fire: u32,
    magnet_field: u32,
    xp_hunger: u32,
    vital_spark: u32,
    core_surge: u32,
    stellar_power: u32,
}

struct LevelChoice {
    offers: [UpgradeKind; 3],
    menu: gotoo_pixel_engine::ui::MenuState,
}

struct VoidCanticleV13 {
    combat: VoidCanticleV12,
    xp_orbs: Vec<XpOrb>,
    xp: u32,
    level: u32,
    xp_next: u32,
    build: BuildState,
    level_choice: Option<LevelChoice>,
    level_controls: ControlMap,
}

impl VoidCanticleV13 {
    fn new() -> Self {
        let level = 1;
        Self {
            combat: VoidCanticleV12::new(),
            xp_orbs: Vec::new(),
            xp: 0,
            level,
            xp_next: xp_requirement(level),
            build: BuildState::default(),
            level_choice: None,
            level_controls: gotoo_pixel_engine::ui::standard_menu_controls(
                VC_LEVEL_UP,
                VC_LEVEL_DOWN,
                VC_LEVEL_CONFIRM,
            ),
        }
    }

    fn reset_progression(&mut self) {
        self.xp_orbs.clear();
        self.xp = 0;
        self.level = 1;
        self.xp_next = xp_requirement(self.level);
        self.build = BuildState::default();
        self.level_choice = None;
    }

    fn reset_run(&mut self) {
        self.combat.reset_run();
        self.reset_progression();
    }

    fn magnet_radius(&self) -> f32 {
        (BASE_MAGNET_RADIUS + MAGNET_RADIUS_PER_STACK * self.build.magnet_field as f32)
            .min(150.0)
    }

    fn xp_gain_percent(&self) -> u32 {
        100_u32
            .saturating_add(XP_GAIN_PER_STACK.saturating_mul(self.build.xp_hunger))
            .min(300)
    }

    fn apply_pre_update_build(&mut self, dt: f32) {
        if self.build.rapid_fire == 0 {
            return;
        }

        let bonus = RAPID_FIRE_PER_STACK * self.build.rapid_fire as f32;
        let base = &mut self.combat.combat.ui.inner.inner.base;
        base.fire_cooldown = (base.fire_cooldown - dt * bonus).max(0.0);
    }

    fn spawn_xp_from_new_kill_bursts(&mut self) {
        let base = &self.combat.combat.ui.inner.inner.base;
        let mut spawned = Vec::new();

        for burst in &base.bursts {
            if (burst.remaining - burst.duration).abs() > 0.0001 {
                continue;
            }

            let value = if (burst.duration - 0.24).abs() < 0.0001 {
                Some(XP_ORB_CARRION)
            } else if (burst.duration - 0.34).abs() < 0.0001 {
                Some(XP_ORB_SPECIAL)
            } else if (burst.duration - 0.40).abs() < 0.0001 {
                Some(XP_ORB_THREAT)
            } else {
                None
            };

            if let Some(value) = value {
                spawned.push(XpOrb::new(burst.x, burst.y, value));
            }
        }

        self.xp_orbs.extend(spawned);
    }

    fn update_xp_orbs(&mut self, dt: f32, frame: &mut Frame<'_>) {
        let player_x = self.combat.combat.ui.inner.inner.base.player_x;
        let player_y = self.combat.combat.ui.inner.inner.base.player_y;
        let magnet_radius = self.magnet_radius();
        let magnet_radius_sq = magnet_radius * magnet_radius;
        let mut collected = 0_u32;

        for orb in &mut self.xp_orbs {
            orb.age += dt;
            let dx = player_x - orb.x;
            let dy = player_y - orb.y;
            let distance_sq = dx * dx + dy * dy;

            if distance_sq <= magnet_radius_sq {
                let distance = distance_sq.sqrt().max(0.001);
                let pull = (115.0 + (magnet_radius - distance).max(0.0) * 3.6).min(360.0);
                orb.vx += dx / distance * pull * dt;
                orb.vy += dy / distance * pull * dt;
                let damping = (1.0 - 4.0 * dt).max(0.0);
                orb.vx *= damping;
                orb.vy *= damping;
            } else {
                orb.vy = (orb.vy + 12.0 * dt).min(30.0);
                orb.vx *= (1.0 - 1.6 * dt).max(0.0);
            }

            orb.x += orb.vx * dt;
            orb.y += orb.vy * dt;

            if point_near(orb.x, orb.y, player_x, player_y, 8.0) {
                orb.alive = false;
                collected = collected.saturating_add(orb.value);
            } else if orb.age > 13.0 || orb.y > FRAMEBUFFER_HEIGHT as f32 + 12.0 {
                orb.alive = false;
            }
        }

        self.xp_orbs.retain(|orb| orb.alive);

        if collected > 0 {
            let gained = collected.saturating_mul(self.xp_gain_percent()) / 100;
            self.xp = self.xp.saturating_add(gained);
            let _ = self
                .combat
                .combat
                .ui
                .inner
                .inner
                .base
                .sounds
                .play(frame.audio, CINDER_SOUND);
            self.maybe_start_level_up(frame);
        }
    }

    fn maybe_start_level_up(&mut self, frame: &mut Frame<'_>) {
        if self.level_choice.is_some() || self.xp < self.xp_next {
            return;
        }

        self.xp -= self.xp_next;
        self.level = self.level.saturating_add(1);
        self.xp_next = xp_requirement(self.level);
        let offers = self.build_offers();
        self.level_choice = Some(LevelChoice {
            offers,
            menu: gotoo_pixel_engine::ui::MenuState::new(3),
        });

        let _ = self
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .sounds
            .play(frame.audio, POWERUP_SOUND);
    }

    fn build_offers(&self) -> [UpgradeKind; 3] {
        let start = (self.level as usize * 2 + self.build.rapid_fire as usize) % UPGRADE_POOL.len();
        let mut offers = [UpgradeKind::CoreSurge; 3];
        let mut count = 0_usize;

        for offset in 0..UPGRADE_POOL.len() * 2 {
            let candidate = UPGRADE_POOL[(start + offset) % UPGRADE_POOL.len()];
            if candidate == UpgradeKind::StellarPower
                && self.combat.combat.ui.inner.inner.power_level >= MAX_POWER_LEVEL
            {
                continue;
            }
            if offers[..count].contains(&candidate) {
                continue;
            }

            offers[count] = candidate;
            count += 1;
            if count == offers.len() {
                break;
            }
        }

        debug_assert_eq!(count, offers.len());
        offers
    }

    fn apply_upgrade(&mut self, upgrade: UpgradeKind, frame: &mut Frame<'_>) {
        match upgrade {
            UpgradeKind::RapidFire => {
                self.build.rapid_fire = self.build.rapid_fire.saturating_add(1);
            }
            UpgradeKind::MagnetField => {
                self.build.magnet_field = self.build.magnet_field.saturating_add(1);
            }
            UpgradeKind::StellarPower => {
                self.build.stellar_power = self.build.stellar_power.saturating_add(1);
                let game = &mut self.combat.combat.ui.inner.inner;
                if game.power_level < MAX_POWER_LEVEL {
                    game.power_level += 1;
                }
            }
            UpgradeKind::XpHunger => {
                self.build.xp_hunger = self.build.xp_hunger.saturating_add(1);
            }
            UpgradeKind::VitalSpark => {
                self.build.vital_spark = self.build.vital_spark.saturating_add(1);
                let base = &mut self.combat.combat.ui.inner.inner.base;
                base.lives = base.lives.saturating_add(1).min(9);
            }
            UpgradeKind::CoreSurge => {
                self.build.core_surge = self.build.core_surge.saturating_add(1);
                let base = &mut self.combat.combat.ui.inner.inner.base;
                base.core_charge = base.core_charge.saturating_add(30).min(CORE_MAX);
            }
        }

        let base = &mut self.combat.combat.ui.inner.inner.base;
        base.bursts.push(Burst::new(
            base.player_x,
            base.player_y - 4.0,
            0.42,
            XP_ORB_CORE,
        ));
        let _ = base.sounds.play(frame.audio, POWERUP_SOUND);
    }

    fn update_level_choice(&mut self, frame: &mut Frame<'_>) {
        let mut selected = None;
        if let Some(choice) = self.level_choice.as_mut() {
            if self.level_controls.action(VC_LEVEL_UP).pressed() {
                choice.menu.select_previous();
            }
            if self.level_controls.action(VC_LEVEL_DOWN).pressed() {
                choice.menu.select_next();
            }
            if self.level_controls.action(VC_LEVEL_CONFIRM).pressed()
                && let Some(index) = choice.menu.selected()
            {
                selected = choice.offers.get(index).copied();
            }
        }

        if let Some(upgrade) = selected {
            self.level_choice = None;
            self.apply_upgrade(upgrade, frame);
            self.maybe_start_level_up(frame);
        }
    }

    fn render_progression(&self, framebuffer: &mut Framebuffer) {
        for orb in &self.xp_orbs {
            let x = orb.x.round() as i32;
            let y = orb.y.round() as i32;
            framebuffer.draw(x - 1, y + 2, XP_ORB);
            framebuffer.fill_circle(x, y, 2, XP_ORB);
            framebuffer.draw(x, y, XP_ORB_CORE);
        }

        if let Some(choice) = &self.level_choice {
            self.render_level_choice(framebuffer, choice);
        }
    }

    fn render_level_choice(&self, framebuffer: &mut Framebuffer, choice: &LevelChoice) {
        let panel = gotoo_pixel_engine::Rect {
            x: 8,
            y: 48,
            width: 164,
            height: 220,
        };
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            panel,
            Pixel::rgb(7, 10, 19),
            XP_ORB,
        );
        framebuffer.draw_text(61, 61, "LEVEL UP", XP_ORB_CORE);
        framebuffer.draw_text(64, 75, &format!("LEVEL {}", self.level), WRECK_LIGHT);

        for (index, upgrade) in choice.offers.iter().copied().enumerate() {
            let y = 98 + index as i32 * 47;
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 18,
                    y,
                    width: 144,
                    height: 18,
                },
                upgrade_name(upgrade),
                choice.menu.selected() == Some(index),
                1,
                TEXT,
                XP_ORB_CORE,
            );
            framebuffer.draw_text(28, y + 23, upgrade_description(upgrade), WRECK_LIGHT);
        }

        framebuffer.draw_text(39, 244, "SPACE SOUTH SELECT", WRECK_LIGHT);
    }
}

impl Game for VoidCanticleV13 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.level_controls.update(frame.input);

        if self.level_choice.is_some() {
            self.update_level_choice(frame);
            self.combat.render(frame.framebuffer, false);
            self.render_progression(frame.framebuffer);
            return GameResult::Continue;
        }

        let was_game_over = self.combat.combat.ui.inner.inner.base.game_over;
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.apply_pre_update_build(dt);

        let result = self.combat.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let game_over = self.combat.combat.ui.inner.inner.base.game_over;
        if was_game_over && !game_over {
            self.reset_progression();
        }

        self.spawn_xp_from_new_kill_bursts();
        self.update_xp_orbs(dt, frame);
        self.render_progression(frame.framebuffer);
        GameResult::Continue
    }
}

fn xp_requirement(level: u32) -> u32 {
    let mut required = 30_u32;
    for _ in 1..level {
        required = required.saturating_mul(134).saturating_add(50) / 100;
    }
    required
}

fn upgrade_name(upgrade: UpgradeKind) -> &'static str {
    match upgrade {
        UpgradeKind::RapidFire => "RAPID FIRE",
        UpgradeKind::MagnetField => "MAGNET FIELD",
        UpgradeKind::StellarPower => "STELLAR POWER",
        UpgradeKind::XpHunger => "XP HUNGER",
        UpgradeKind::VitalSpark => "VITAL SPARK",
        UpgradeKind::CoreSurge => "CORE SURGE",
    }
}

fn upgrade_description(upgrade: UpgradeKind) -> &'static str {
    match upgrade {
        UpgradeKind::RapidFire => "FIRE RATE +18",
        UpgradeKind::MagnetField => "PICKUP RANGE +22",
        UpgradeKind::StellarPower => "POWER LEVEL +1",
        UpgradeKind::XpHunger => "XP VALUE +25",
        UpgradeKind::VitalSpark => "LIFE +1",
        UpgradeKind::CoreSurge => "CORE +30",
    }
}

struct VoidCanticlePauseV13 {
    game: VoidCanticleV13,
    state: VcPauseState,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
}

impl VoidCanticlePauseV13 {
    fn new(game: VoidCanticleV13) -> Self {
        let mut controls = gotoo_pixel_engine::ui::standard_menu_controls(
            VC_PAUSE_UP,
            VC_PAUSE_DOWN,
            VC_PAUSE_CONFIRM,
        );
        controls
            .bind_key(VC_PAUSE_TOGGLE, Key::Escape)
            .bind_gamepad(VC_PAUSE_TOGGLE, GamepadButton::Start);
        Self {
            game,
            state: VcPauseState::Running,
            menu: gotoo_pixel_engine::ui::MenuState::new(5),
            controls,
        }
    }

    fn pause_input_held(&self) -> bool {
        [VC_PAUSE_TOGGLE, VC_PAUSE_UP, VC_PAUSE_DOWN, VC_PAUSE_CONFIRM]
            .into_iter()
            .any(|action| self.controls.action(action).held())
    }

    fn render_menu(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 18,
                y: 48,
                width: 144,
                height: 220,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(60, 62, "PAUSED", POWER_RELIC_LIGHT);

        for (index, (y, label)) in [
            (90, "RESUME"),
            (118, "RESTART"),
            (146, "CONTROLS"),
            (174, "BUILD INFO"),
            (202, "QUIT"),
        ]
        .into_iter()
        .enumerate()
        {
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 34,
                    y,
                    width: 112,
                    height: 18,
                },
                label,
                self.menu.selected() == Some(index),
                1,
                TEXT,
                POWER_RELIC_LIGHT,
            );
        }
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 10,
                y: 46,
                width: 160,
                height: 224,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(52, 59, "CONTROLS", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 86, "MOVE  ARROWS WASD", TEXT);
        framebuffer.draw_text(20, 106, "FIRE  SPACE SOUTH", TEXT);
        framebuffer.draw_text(20, 126, "FOCUS SHIFT LB", TEXT);
        framebuffer.draw_text(20, 146, "CANTICLE X EAST", TEXT);
        framebuffer.draw_text(20, 166, "PAUSE ESC START", TEXT);
        framebuffer.draw_text(31, 231, "ESC START  BACK", WRECK_LIGHT);
    }

    fn render_build_info(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 10,
                y: 58,
                width: 160,
                height: 196,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(46, 72, "BUILD INFO", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 103, &format!("VERSION {VC13_VERSION}"), TEXT);
        framebuffer.draw_text(20, 123, &format!("BUILD {BUILD_ID}"), TEXT);
        framebuffer.draw_text(20, 143, "STAGE GRAVE ORBIT", TEXT);
        framebuffer.draw_text(20, 163, "GPE DEV BUILD", WRECK_LIGHT);
        framebuffer.draw_text(31, 219, "ESC START  BACK", WRECK_LIGHT);
    }
}

impl Game for VoidCanticlePauseV13 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.controls.update(frame.input);

        match self.state {
            VcPauseState::Running => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed() {
                    self.state = VcPauseState::Menu;
                    self.menu = gotoo_pixel_engine::ui::MenuState::new(5);
                    self.render_menu(frame.framebuffer);
                    GameResult::Continue
                } else {
                    self.game.update(frame)
                }
            }
            VcPauseState::Menu => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed() {
                    self.state = VcPauseState::ResumeGate;
                } else {
                    if self.controls.action(VC_PAUSE_UP).pressed() {
                        self.menu.select_previous();
                    }
                    if self.controls.action(VC_PAUSE_DOWN).pressed() {
                        self.menu.select_next();
                    }
                    if self.controls.action(VC_PAUSE_CONFIRM).pressed() {
                        match self.menu.selected() {
                            Some(0) => self.state = VcPauseState::ResumeGate,
                            Some(1) => {
                                self.game.reset_run();
                                self.state = VcPauseState::ResumeGate;
                            }
                            Some(2) => self.state = VcPauseState::Controls,
                            Some(3) => self.state = VcPauseState::BuildInfo,
                            Some(4) => return GameResult::Exit,
                            _ => {}
                        }
                    }
                }

                match self.state {
                    VcPauseState::Controls => self.render_controls(frame.framebuffer),
                    VcPauseState::BuildInfo => self.render_build_info(frame.framebuffer),
                    _ => self.render_menu(frame.framebuffer),
                }
                GameResult::Continue
            }
            VcPauseState::Controls => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed()
                    || self.controls.action(VC_PAUSE_CONFIRM).pressed()
                {
                    self.state = VcPauseState::Menu;
                    self.render_menu(frame.framebuffer);
                } else {
                    self.render_controls(frame.framebuffer);
                }
                GameResult::Continue
            }
            VcPauseState::BuildInfo => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed()
                    || self.controls.action(VC_PAUSE_CONFIRM).pressed()
                {
                    self.state = VcPauseState::Menu;
                    self.render_menu(frame.framebuffer);
                } else {
                    self.render_build_info(frame.framebuffer);
                }
                GameResult::Continue
            }
            VcPauseState::ResumeGate => {
                if self.pause_input_held() {
                    self.render_menu(frame.framebuffer);
                } else {
                    self.state = VcPauseState::Running;
                }
                GameResult::Continue
            }
        }
    }
}

pub fn run_v13_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: "Void Canticle - Gotoo Pixel Engine".to_string(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticlePauseV13::new(VoidCanticleV13::new()),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v13_tests {
    use super::*;

    #[test]
    fn xp_curve_grows_geometrically() {
        assert_eq!(xp_requirement(1), 30);
        assert!(xp_requirement(2) > xp_requirement(1));
        assert!(xp_requirement(4) > xp_requirement(2));
        assert!(xp_requirement(6) > xp_requirement(1) * 4);
    }

    #[test]
    fn magnet_upgrade_increases_pickup_radius() {
        let mut game = VoidCanticleV13::new();
        let base = game.magnet_radius();
        game.build.magnet_field = 2;
        assert!(game.magnet_radius() > base + 40.0);
    }

    #[test]
    fn xp_hunger_increases_orb_value() {
        let mut game = VoidCanticleV13::new();
        assert_eq!(game.xp_gain_percent(), 100);
        game.build.xp_hunger = 2;
        assert_eq!(game.xp_gain_percent(), 150);
    }

    #[test]
    fn first_level_offers_three_distinct_upgrades() {
        let game = VoidCanticleV13::new();
        let offers = game.build_offers();
        assert_ne!(offers[0], offers[1]);
        assert_ne!(offers[0], offers[2]);
        assert_ne!(offers[1], offers[2]);
    }

    #[test]
    fn vc13_version_is_explicit() {
        assert_eq!(VC13_VERSION, "VC1.3");
    }
}
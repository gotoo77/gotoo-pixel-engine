const VC20_VERSION: &str = "VC2.0";

struct VoidCanticleV20 {
    inner: VoidCanticleV19,
    last_gameplay_frame: Option<Framebuffer>,
    pause_snapshot: Option<Framebuffer>,
}

impl VoidCanticleV20 {
    fn new() -> Self {
        Self {
            inner: VoidCanticleV19::new(),
            last_gameplay_frame: None,
            pause_snapshot: None,
        }
    }

    fn base(&self) -> &VoidCanticleGame {
        self.inner.base()
    }

    fn restore_strip(
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        sample_y: i32,
    ) {
        for dx in 0..width {
            let px = x + dx as i32;
            let sample = framebuffer.pixel(px, sample_y).unwrap_or(BG);
            for dy in 0..height {
                framebuffer.draw(px, y + dy as i32, sample);
            }
        }
    }

    fn remove_matching_backing(
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        backing: Pixel,
        sample_y: i32,
    ) {
        for dx in 0..width {
            let px = x + dx as i32;
            let sample = framebuffer.pixel(px, sample_y).unwrap_or(BG);
            for dy in 0..height {
                let py = y + dy as i32;
                if framebuffer.pixel(px, py) == Some(backing) {
                    framebuffer.draw(px, py, sample);
                }
            }
        }
    }

    fn render_clean_hud_bars(&self, framebuffer: &mut Framebuffer) {
        // Boss bar: keep only the live fill, two pixels high, over the scene.
        Self::restore_strip(framebuffer, 24, 15, 132, 5, 13);
        if let Some(boss) = self.base().boss
            && self.base().encounter_phase == EncounterPhase::BossFight
        {
            let width = 132_u32.saturating_mul(boss.hp) / BELLKEEPER_MAX_HP;
            if width > 0 {
                framebuffer.fill_rect(24, 16, width, 2, DANGER);
            }
        }

        // CORE: erase the historical opaque trough, then draw only the charge.
        Self::restore_strip(framebuffer, 34, 303, 76, 5, 299);
        let core_width = 76_u32.saturating_mul(self.base().core_charge) / CORE_MAX;
        if core_width > 0 {
            let core_color = if self.base().core_charge >= CORE_MAX {
                CANTICLE_COLOR
            } else {
                CINDER
            };
            framebuffer.fill_rect(34, 304, core_width, 2, core_color);
        }

        // Power pips: active pips only; inactive black placeholders disappear.
        Self::restore_strip(framebuffer, 143, 303, 33, 5, 299);
        let power_level = self
            .inner
            .ui
            .game
            .combat
            .combat
            .combat
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .power_level;
        for index in 0..MAX_POWER_LEVEL {
            if index < power_level {
                framebuffer.fill_rect(143 + i32::from(index) * 7, 304, 5, 2, POWER_RELIC);
            }
        }

        // XP: same treatment as CORE; no full-width dark strip underneath.
        Self::restore_strip(framebuffer, 4, 315, 172, 3, 312);
        let progression = &self.inner.v14().progression;
        let xp_width = 172_u32
            .saturating_mul(progression.xp.min(progression.xp_next))
            .checked_div(progression.xp_next)
            .unwrap_or(172);
        if xp_width > 0 {
            framebuffer.fill_rect(4, 315, xp_width, 2, XP_BAR_FILL);
        }

        // The compact VOID status should float too, not become a black plaque.
        Self::remove_matching_backing(
            framebuffer,
            116,
            288,
            60,
            11,
            Pixel::rgb(6, 8, 17),
            286,
        );
    }

    fn render_build_info_overlay(&self, framebuffer: &mut Framebuffer) {
        framebuffer.fill_rect(17, 92, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 97, &format!("VERSION {VC20_VERSION}"), TEXT);
        framebuffer.fill_rect(17, 172, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 177, "HUD CLEANUP PASS", ART_GOLD);
    }

    fn render_pause_overlay(&self, framebuffer: &mut Framebuffer) {
        match self.inner.ui.state {
            VcPauseState::Running => {}
            VcPauseState::Menu | VcPauseState::ResumeGate => {
                self.inner.ui.render_menu(framebuffer);
            }
            VcPauseState::Controls => {
                self.inner.ui.render_controls(framebuffer);
            }
            VcPauseState::BuildInfo => {
                self.inner.ui.render_build_info(framebuffer);
                self.render_build_info_overlay(framebuffer);
            }
        }
    }

    fn restore_pause_snapshot(&self, framebuffer: &mut Framebuffer) -> bool {
        let Some(snapshot) = &self.pause_snapshot else {
            return false;
        };
        framebuffer.clone_from(snapshot);
        true
    }

    fn remember_gameplay_frame(&mut self, framebuffer: &Framebuffer) {
        if let Some(snapshot) = &mut self.last_gameplay_frame {
            snapshot.clone_from(framebuffer);
        } else {
            self.last_gameplay_frame = Some(framebuffer.clone());
        }
    }
}

impl Game for VoidCanticleV20 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let state_before = self.inner.ui.state;

        let result = self.inner.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let state_after = self.inner.ui.state;
        let was_running = matches!(state_before, VcPauseState::Running);
        let is_running = matches!(state_after, VcPauseState::Running);

        if was_running && !is_running {
            self.pause_snapshot = self.last_gameplay_frame.clone();
        }

        if is_running {
            // ResumeGate intentionally does not advance simulation. Restore the frozen
            // gameplay image once so the pause panel cannot survive for a frame.
            if matches!(state_before, VcPauseState::ResumeGate) {
                self.restore_pause_snapshot(frame.framebuffer);
            }

            self.render_clean_hud_bars(frame.framebuffer);
            self.remember_gameplay_frame(frame.framebuffer);
            self.pause_snapshot = None;
            return GameResult::Continue;
        }

        // Every paused frame starts from the same pristine gameplay snapshot. This
        // prevents Menu / Controls / Build Info panels from painting over each other.
        self.restore_pause_snapshot(frame.framebuffer);
        self.render_clean_hud_bars(frame.framebuffer);
        self.render_pause_overlay(frame.framebuffer);

        GameResult::Continue
    }
}

pub fn run_v20_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticleV20::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v20_tests {
    use super::*;

    #[test]
    fn cleanup_removes_opaque_bottom_troughs() {
        let game = VoidCanticleV20::new();
        let mut framebuffer = Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        framebuffer.clear(BG);
        framebuffer.fill_rect(34, 303, 76, 5, CORE_BG);
        framebuffer.fill_rect(4, 315, 172, 3, XP_BAR_BG);

        game.render_clean_hud_bars(&mut framebuffer);

        assert_ne!(framebuffer.pixel(80, 307), Some(CORE_BG));
        assert_ne!(framebuffer.pixel(100, 317), Some(XP_BAR_BG));
    }

    #[test]
    fn v20_version_is_explicit() {
        assert_eq!(VC20_VERSION, "VC2.0");
    }
}

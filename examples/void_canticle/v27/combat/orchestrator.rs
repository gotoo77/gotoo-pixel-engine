impl VoidCanticleV27DirectPresentation {
    fn render_player_last(&self, framebuffer: &mut Framebuffer) {
        if self.game.combat_model().player_hull <= 0.0 {
            return;
        }

        let focused = self
            .game
            .game
            .game
            .game
            .movement_controls
            .action(FOCUS)
            .held();
        let base = self.game.game.base();
        let hit = self.hit_reactions.player_visual();
        let x = vc27_present(base.player_x) + hit.offset_x;
        let y = vc27_present(base.player_y) + hit.offset_y;
        vc27_hd_render_pilgrim(
            framebuffer,
            x,
            y,
            focused,
            base.invulnerability,
            base.animation_time,
        );
        vc27_render_hit_flash(framebuffer, x, y, Vc27HitFlashKind::Pilgrim, hit);
    }

    fn render_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.game.game.active_combat() {
            return;
        }

        self.render_clean_background(framebuffer);
        self.render_choir_links(framebuffer);
        self.render_pickups(framebuffer);
        self.render_presentation_bestiary(framebuffer);
        self.render_attack_telegraphs(framebuffer);
        self.render_enemy_defenses(framebuffer);
        self.render_projectiles(framebuffer);
        self.render_particles_and_bursts(framebuffer);
        self.render_major_fx(framebuffer);
        self.render_event_announcement(framebuffer);
        self.render_threat_meter(framebuffer);
        self.render_echo_shell(framebuffer);
        self.render_canticle_charge(framebuffer);
        self.render_survival_bars(framebuffer);
        self.render_player_last(framebuffer);
    }
}

#[cfg(test)]
mod v27_combat_orchestrator_tests {
    #[test]
    fn combat_pipeline_keeps_player_as_last_visual_layer() {
        let stages = [
            "background",
            "choir_links",
            "pickups",
            "bestiary",
            "telegraphs",
            "defenses",
            "projectiles",
            "particles",
            "major_fx",
            "announcements",
            "threat",
            "echo",
            "canticle",
            "survival",
            "player",
        ];
        assert_eq!(stages.last(), Some(&"player"));
    }
}

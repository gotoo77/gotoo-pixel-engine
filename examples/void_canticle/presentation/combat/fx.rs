impl VoidCanticlePresentation {
    fn render_particles_and_bursts(&self, framebuffer: &mut Framebuffer) {
        for burst in &self.game.presentation_base().bursts {
            let ratio = if burst.duration > 0.0 {
                (burst.remaining / burst.duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let radius = (3.0 + (1.0 - ratio) * 16.0).round() as u32;
            framebuffer.draw_circle(
                vc27_present(burst.x),
                vc27_present(burst.y),
                radius,
                burst.color,
            );
        }

        for particle in self.game.presentation_particles() {
            let x = vc27_present(particle.x);
            let y = vc27_present(particle.y);
            match particle.kind {
                V17ParticleKind::Spark => {
                    let tail_x = vc27_present(particle.x - particle.vx * 0.035);
                    let tail_y = vc27_present(particle.y - particle.vy * 0.035);
                    framebuffer.draw_line(tail_x, tail_y, x, y, particle.color);
                    framebuffer.draw(x, y, BOLT_CORE);
                }
                V17ParticleKind::Shard => {
                    framebuffer.draw_line(x - 2, y, x + 2, y, particle.color);
                    framebuffer.draw_line(x, y - 2, x, y + 2, particle.color);
                }
            }
        }
    }

    fn render_major_fx(&self, framebuffer: &mut Framebuffer) {
        let base = self.game.presentation_base();
        let player_x = vc27_present(base.player_x);
        let player_y = vc27_present(base.player_y);

        if base.canticle_timer > 0.0 {
            let ratio = (base.canticle_timer / CANTICLE_DURATION).clamp(0.0, 1.0);
            let radius = (40.0 + (1.0 - ratio) * 150.0).round() as u32;
            framebuffer.draw_circle(player_x, player_y, radius, CANTICLE_COLOR);
            framebuffer.draw_circle(player_x, player_y, radius.saturating_add(12), ART_GOLD);
        }

        let emp_timer = self.game.presentation_emp_flash_timer();
        if emp_timer > 0.0 {
            let ratio = (emp_timer / VC23_EMP_FLASH_DURATION).clamp(0.0, 1.0);
            let radius = (28.0 + (1.0 - ratio) * 220.0).round() as u32;
            framebuffer.draw_circle(player_x, player_y, radius, ART_CYAN_LIGHT);
        }
    }
}

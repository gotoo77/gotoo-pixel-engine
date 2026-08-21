// Narrow semantic seam consumed by the current presentation layer.
//
// Historical versioned types, constants, and wrapper traversal are allowed
// here because this file lives inside the legacy quarantine. Current
// presentation code must use the semantic names below instead of knowing which
// historical layer currently owns the data.

type GameplayRuntime = VoidCanticleV23Sustain;
type ChassisSelector = VoidCanticleV22;
type SustainAugment = Vc23SustainAugment;
type AttackSound = Vc23AttackSound;
type CombatParticleKind = V17ParticleKind;

const CHASSIS_OPTIONS: [ExosuitChassis; 3] = VC22_CHASSIS;
const SUSTAIN_OPTIONS: [SustainAugment; 2] = VC23_SUSTAIN_AUGMENTS;
const NANITE_DELAY: f32 = VC23_NANITE_DELAY;
const NANITE_REPAIR_PER_SECOND: f32 = VC23_NANITE_REPAIR_PER_SECOND;
const CAPACITOR_DELAY: f32 = VC23_CAPACITOR_DELAY;
const CAPACITOR_REGEN_PER_SECOND: f32 = VC23_CAPACITOR_REGEN_PER_SECOND;
const PRESENTATION_HULL_COLOR: Pixel = VC20_HULL;
const PRESENTATION_ARMOR_COLOR: Pixel = VC20_ARMOR;
const PRESENTATION_ARMOR_LIGHT: Pixel = VC20_ARMOR_LIGHT;
const PRESENTATION_ARMOR_BG: Pixel = VC20_ARMOR_BG;
const PRESENTATION_BOSS_SHIELD_MAX: u32 = VC20_BOSS_SHIELD_MAX;
const PRESENTATION_EMP_FLASH_DURATION: f32 = VC23_EMP_FLASH_DURATION;
const PRESENTATION_CARRION_FIRE_SOUND: SoundId = VC23_CARRION_FIRE_SOUND;
const PRESENTATION_WRAITH_FIRE_SOUND: SoundId = VC23_WRAITH_FIRE_SOUND;
const PRESENTATION_VOID_PULSE_FIRE_SOUND: SoundId = VC23_VOID_PULSE_FIRE_SOUND;
const PRESENTATION_VOID_FIRE_SOUND: SoundId = VC23_VOID_FIRE_SOUND;
const PRESENTATION_BELLKEEPER_FIRE_SOUND: SoundId = VC23_BELLKEEPER_FIRE_SOUND;

fn presentation_void_attack_speed(speed: f32) -> bool {
    vc23_void_attack_speed(speed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresentationAnnouncement {
    Pressure(VoidPressure),
    BossPhase(BellPhase),
    Synergy(&'static str),
    Canticle,
    VoidAttack(VoidAttackKind),
}

impl GameplayRuntime {
    fn presentation_base(&self) -> &VoidCanticleGame {
        self.game.base()
    }

    fn presentation_active_combat(&self) -> bool {
        self.game.active_combat()
    }

    fn presentation_progression(&self) -> &VoidCanticleV14 {
        self.game.v20().game.v14()
    }

    fn presentation_defense_model(&self) -> &VoidCanticleV20 {
        self.game.v20()
    }

    fn presentation_encounter_model(&self) -> &VoidCanticleV12 {
        self.presentation_defense_model().game.v12()
    }

    fn presentation_particle_layer(&self) -> &VoidCanticleV17 {
        &self.presentation_defense_model().game.ui.game
    }

    fn presentation_particles(&self) -> &[V17Particle] {
        &self.presentation_particle_layer().particles
    }

    fn presentation_event_layer(&self) -> &VoidCanticleV16B {
        &self.presentation_particle_layer().combat
    }

    fn presentation_void_pressure(&self) -> VoidPressure {
        self.presentation_event_layer().combat.pressure
    }

    fn presentation_pending_void_attack_kind(&self) -> Option<VoidAttackKind> {
        self.presentation_event_layer()
            .combat
            .pending_attack
            .as_ref()
            .map(|attack| attack.kind)
    }

    fn presentation_announcement(&self) -> Option<PresentationAnnouncement> {
        let events = self.presentation_event_layer();
        if events.pressure_reveal_timer > 0.0 && events.pressure_reveal != VoidPressure::Dormant {
            return Some(PresentationAnnouncement::Pressure(events.pressure_reveal));
        }
        if events.boss_phase_banner_timer > 0.0
            && let Some(phase) = events.boss_phase_banner
        {
            return Some(PresentationAnnouncement::BossPhase(phase));
        }

        let combat = &events.combat.combat;
        if combat.synergy_banner_timer > 0.0
            && let Some(name) = combat.synergy_banner_name
        {
            return Some(PresentationAnnouncement::Synergy(name));
        }
        if self.presentation_base().canticle_timer > 0.0 {
            return Some(PresentationAnnouncement::Canticle);
        }
        events
            .combat
            .pending_attack
            .as_ref()
            .map(|attack| PresentationAnnouncement::VoidAttack(attack.kind))
    }

    fn presentation_emp_flash_timer(&self) -> f32 {
        self.game.emp_flash_timer
    }

    fn presentation_chassis_selector(&self) -> &ChassisSelector {
        &self.game.game.game.game
    }

    fn presentation_chassis_selection_active(&self) -> bool {
        self.presentation_chassis_selector().chassis.is_none()
    }

    fn presentation_selected_chassis_choice(&self) -> Option<(usize, ExosuitChassis)> {
        let selector = self.presentation_chassis_selector();
        let index = selector.menu.selected()?;
        let chassis = CHASSIS_OPTIONS.get(index).copied()?;
        Some((index, chassis))
    }

    fn presentation_reset_chassis_selection(&mut self) {
        self.game
            .game
            .game
            .game
            .reset_chassis_selection_for_new_run();
    }

    fn presentation_support_choice_active(&self) -> bool {
        self.choosing_support
    }

    fn presentation_selected_support_choice(&self) -> Option<(usize, SustainAugment)> {
        let index = self.menu.selected()?;
        let augment = SUSTAIN_OPTIONS.get(index).copied()?;
        Some((index, augment))
    }

    fn presentation_focus_held(&self) -> bool {
        self.game
            .game
            .game
            .movement_controls
            .action(FOCUS)
            .held()
    }

    fn presentation_selected_chassis(&self) -> Option<ExosuitChassis> {
        self.presentation_chassis_selector().chassis
    }

    fn presentation_chassis_confirm_armed(&self) -> bool {
        self.presentation_chassis_selector().confirm_armed
    }

    #[cfg(test)]
    fn presentation_apply_chassis_for_test(&mut self, chassis: ExosuitChassis) {
        self.game.game.game.game.apply_chassis(chassis);
    }

    #[cfg(test)]
    fn presentation_arm_chassis_confirm_for_test(&mut self) {
        self.game.game.game.game.confirm_armed = true;
    }

    #[cfg(test)]
    fn presentation_select_next_chassis_for_test(&mut self) {
        self.game.game.game.game.menu.select_next();
    }
}

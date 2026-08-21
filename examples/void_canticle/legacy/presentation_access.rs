// Narrow semantic seam consumed by the current presentation layer.
//
// Historical versioned types and wrapper traversal are allowed here because
// this file lives inside the legacy quarantine. Current presentation code must
// ask semantic questions through these methods instead of knowing the nested
// vXX storage shape.

impl VoidCanticleV23Sustain {
    fn presentation_base(&self) -> &VoidCanticleGame {
        self.game.base()
    }

    fn presentation_active_combat(&self) -> bool {
        self.game.active_combat()
    }

    fn presentation_progression(&self) -> &VoidCanticleV14 {
        self.game.v20().game.v14()
    }

    fn presentation_chassis_selector(&self) -> &VoidCanticleV22 {
        &self.game.game.game.game
    }

    fn presentation_chassis_selection_active(&self) -> bool {
        self.presentation_chassis_selector().chassis.is_none()
    }

    fn presentation_selected_chassis_choice(&self) -> Option<(usize, ExosuitChassis)> {
        let selector = self.presentation_chassis_selector();
        let index = selector.menu.selected()?;
        let chassis = VC22_CHASSIS.get(index).copied()?;
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

    fn presentation_selected_support_choice(&self) -> Option<(usize, Vc23SustainAugment)> {
        let index = self.menu.selected()?;
        let augment = VC23_SUSTAIN_AUGMENTS.get(index).copied()?;
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

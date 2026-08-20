fn vc27_chassis_profile(chassis: ExosuitChassis) -> Vc27ChoiceProfile<'static> {
    let (accent, renderer) = match chassis {
        ExosuitChassis::Bulwark => (
            ART_GOLD,
            vc27_chassis_bulwark_art as Vc27ChoiceArtRenderer,
        ),
        ExosuitChassis::Pilgrim => (
            PILGRIM_VIOLET,
            vc27_chassis_pilgrim_art as Vc27ChoiceArtRenderer,
        ),
        ExosuitChassis::Wraith => (
            ART_CYAN_LIGHT,
            vc27_chassis_wraith_art as Vc27ChoiceArtRenderer,
        ),
    };

    Vc27ChoiceProfile::new(
        chassis.name(),
        chassis.role(),
        accent,
        Vc27ChoiceAssets::procedural(renderer),
    )
}

fn vc27_chassis_bulwark_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    vc27_render_chassis_ship(
        framebuffer,
        ExosuitChassis::Bulwark,
        x,
        y,
        selected,
        time,
    );
}

fn vc27_chassis_pilgrim_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    vc27_render_chassis_ship(
        framebuffer,
        ExosuitChassis::Pilgrim,
        x,
        y,
        selected,
        time,
    );
}

fn vc27_chassis_wraith_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    vc27_render_chassis_ship(
        framebuffer,
        ExosuitChassis::Wraith,
        x,
        y,
        selected,
        time,
    );
}

#[cfg(test)]
mod chassis_profile_tests {
    use super::*;

    #[test]
    fn chassis_profiles_carry_the_same_audio_contract_as_other_choices() {
        for chassis in VC22_CHASSIS {
            let profile = vc27_chassis_profile(chassis);
            assert_eq!(profile.hover_sound(), Some(VC27_CHOICE_HOVER_SOUND));
            assert_eq!(profile.confirm_sound(), Some(VC27_CHOICE_CONFIRM_SOUND));
        }
    }

    #[test]
    fn chassis_profiles_keep_gameplay_names_and_roles_authoritative() {
        for chassis in VC22_CHASSIS {
            let profile = vc27_chassis_profile(chassis);
            assert_eq!(profile.label(), chassis.name());
            assert_eq!(profile.category(), chassis.role());
        }
    }
}

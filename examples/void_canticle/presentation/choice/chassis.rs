fn chassis_profile(chassis: ExosuitChassis) -> ChoiceProfile<'static> {
    let (accent, renderer) = match chassis {
        ExosuitChassis::Bulwark => (
            ART_GOLD,
            chassis_bulwark_art as ChoiceArtRenderer,
        ),
        ExosuitChassis::Pilgrim => (
            PILGRIM_VIOLET,
            chassis_pilgrim_art as ChoiceArtRenderer,
        ),
        ExosuitChassis::Wraith => (
            ART_CYAN_LIGHT,
            chassis_wraith_art as ChoiceArtRenderer,
        ),
    };

    ChoiceProfile::new(
        chassis.name(),
        chassis.role(),
        accent,
        ChoiceAssets::procedural(renderer),
    )
}

fn chassis_bulwark_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    render_chassis_ship(
        framebuffer,
        ExosuitChassis::Bulwark,
        x,
        y,
        selected,
        time,
    );
}

fn chassis_pilgrim_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    render_chassis_ship(
        framebuffer,
        ExosuitChassis::Pilgrim,
        x,
        y,
        selected,
        time,
    );
}

fn chassis_wraith_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    render_chassis_ship(
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
        for chassis in CHASSIS_OPTIONS {
            let profile = chassis_profile(chassis);
            let (expected_hover, expected_confirm) = match chassis {
                ExosuitChassis::Bulwark => (
                    Some(ChoiceArtId::Bulwark.hover_override_sound()),
                    Some(ChoiceArtId::Bulwark.confirm_override_sound()),
                ),
                _ => (
                    Some(CHOICE_HOVER_SOUND),
                    Some(CHOICE_CONFIRM_SOUND),
                ),
            };
            assert_eq!(profile.hover_sound(), expected_hover);
            assert_eq!(profile.confirm_sound(), expected_confirm);
        }
    }

    #[test]
    fn chassis_profiles_keep_gameplay_names_and_roles_authoritative() {
        for chassis in CHASSIS_OPTIONS {
            let profile = chassis_profile(chassis);
            assert_eq!(profile.label(), chassis.name());
            assert_eq!(profile.category(), chassis.role());
        }
    }
}

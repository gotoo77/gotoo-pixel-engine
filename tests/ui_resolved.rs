//! External consumer: all geometry, input and assertions use public exports.
use gotoo_pixel_engine::{
    ActionId, Framebuffer, Pixel, Rect, Touch, TouchPhase,
    ui::resolved::{
        self, Activation, ResolvedError, ResolvedState, ResolvedTarget, UiId, UiInput, UiNavInput,
        UiPointerInput,
    },
};

fn id(value: u64) -> UiId {
    UiId::from_stable(value).unwrap()
}
fn target(value: u64, rect: Rect) -> ResolvedTarget {
    ResolvedTarget {
        id: id(value),
        rect,
        action: ActionId::new("select"),
        enabled: true,
    }
}
fn scene() -> [ResolvedTarget; 5] {
    [
        target(
            1,
            Rect {
                x: 90,
                y: 80,
                width: 40,
                height: 30,
            },
        ),
        target(
            2,
            Rect {
                x: 96,
                y: 10,
                width: 28,
                height: 20,
            },
        ),
        target(
            3,
            Rect {
                x: 180,
                y: 75,
                width: 60,
                height: 40,
            },
        ),
        target(
            4,
            Rect {
                x: 80,
                y: 160,
                width: 60,
                height: 50,
            },
        ),
        target(
            5,
            Rect {
                x: 10,
                y: 85,
                width: 30,
                height: 20,
            },
        ),
    ]
}

// Exact rectangles painted by examples/gpe_ui_resolved.rs, deliberately skewed.
fn probe_scene(down_enabled: bool) -> [ResolvedTarget; 5] {
    let mut targets = [
        target(
            1,
            Rect {
                x: 165,
                y: 110,
                width: 100,
                height: 45,
            },
        ),
        target(
            2,
            Rect {
                x: 180,
                y: 45,
                width: 75,
                height: 30,
            },
        ),
        target(
            3,
            Rect {
                x: 330,
                y: 100,
                width: 125,
                height: 55,
            },
        ),
        target(
            4,
            Rect {
                x: 150,
                y: 195,
                width: 130,
                height: 60,
            },
        ),
        target(
            5,
            Rect {
                x: 25,
                y: 115,
                width: 80,
                height: 35,
            },
        ),
    ];
    targets[3].enabled = down_enabled;
    targets
}

#[test]
fn probe_geometry_prefers_direction_cone_without_activation() {
    for down_enabled in [false, true] {
        for (direction, opposite, expected) in [
            (
                UiNavInput {
                    up: true,
                    ..Default::default()
                },
                UiNavInput {
                    down: true,
                    ..Default::default()
                },
                2,
            ),
            (
                UiNavInput {
                    right: true,
                    ..Default::default()
                },
                UiNavInput {
                    left: true,
                    ..Default::default()
                },
                3,
            ),
            (
                UiNavInput {
                    down: true,
                    ..Default::default()
                },
                UiNavInput {
                    up: true,
                    ..Default::default()
                },
                if down_enabled { 4 } else { 1 },
            ),
            (
                UiNavInput {
                    left: true,
                    ..Default::default()
                },
                UiNavInput {
                    right: true,
                    ..Default::default()
                },
                5,
            ),
        ] {
            let targets = probe_scene(down_enabled);
            let mut state = ResolvedState::default();
            let out = step(&mut state, nav(direction), &targets);
            assert_eq!(
                out.focused,
                Some(id(expected)),
                "down={down_enabled}, direction={direction:?}"
            );
            assert_eq!(out.activation, None);
            if expected != 1 {
                let out = step(&mut state, nav(opposite), &targets);
                // Right->Left selects Up: it is in-cone and 2px nearer on
                // the primary axis than Center. The policy is not reciprocal.
                assert_eq!(out.focused, Some(id(if expected == 3 { 2 } else { 1 })));
                assert_eq!(out.activation, None);
            }
        }
    }
}

#[test]
fn cone_boundary_fallback_disabled_and_reorder() {
    let center = target(
        1,
        Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        },
    );
    let boundary = target(
        2,
        Rect {
            x: 40,
            y: 40,
            width: 10,
            height: 10,
        },
    );
    let outside = target(
        3,
        Rect {
            x: 20,
            y: 60,
            width: 10,
            height: 10,
        },
    );
    let right = nav(UiNavInput {
        right: true,
        ..Default::default()
    });
    for (targets, expected) in [
        (vec![center, outside, boundary], 2),
        (vec![center, boundary, outside], 2),
        (vec![center, outside], 3),
        (
            vec![
                center,
                outside,
                ResolvedTarget {
                    enabled: false,
                    ..boundary
                },
            ],
            3,
        ),
    ] {
        let out = step(&mut ResolvedState::default(), right, &targets);
        assert_eq!(out.focused, Some(id(expected)));
        assert_eq!(out.activation, None);
    }
}

#[test]
fn public_input_lifecycle_interrupts_mouse_and_touch_even_between_updates() {
    use gotoo_pixel_engine::Input;
    for touch_capture in [false, true] {
        for gain_before_update in [false, true] {
            let targets = scene();
            let mut input = Input::default();
            let mut state = ResolvedState::default();
            assert!(state.synchronize_input(&input));
            if touch_capture {
                step(
                    &mut state,
                    touches(&[touch(7, TouchPhase::Started, Some((100, 90)))]),
                    &targets,
                );
            } else {
                step(&mut state, pointer(Some((100, 90)), true, false), &targets);
            }
            input.set_window_focused(false);
            if gain_before_update {
                input.set_window_focused(true);
            }
            // A hosted clone must carry both actual focus and the durable reset.
            assert!(!state.synchronize_input(&input.clone()));
            let out = step(&mut state, confirm(), &targets);
            assert!(!out.active(id(1)));
            assert_eq!(out.activation, None);
            assert_eq!(out.focused, Some(id(1)));
            if !gain_before_update {
                for _ in 0..2 {
                    assert!(!state.synchronize_input(&input));
                    assert_eq!(
                        step(&mut state, UiInput::default(), &targets).activation,
                        None
                    );
                }
                input.set_window_focused(true);
            }
            assert!(!state.synchronize_input(&input));
            let inherited = [touch(7, TouchPhase::Ended, Some((100, 90)))];
            let mut end = pointer(Some((100, 90)), false, true);
            end.touches = &inherited;
            end.nav.confirm = true;
            assert_eq!(step(&mut state, end, &targets).activation, None);
            assert!(!state.synchronize_input(&input));
            assert_eq!(
                step(&mut state, UiInput::default(), &targets).activation,
                None
            );
            assert!(state.synchronize_input(&input));
            // Late inherited ends, even after neutral, cannot resurrect captures.
            assert_eq!(step(&mut state, end, &targets).activation, None);
            step(&mut state, pointer(Some((100, 90)), true, false), &targets);
            assert_eq!(
                step(&mut state, pointer(Some((100, 90)), false, true), &targets)
                    .activation
                    .unwrap()
                    .id,
                id(1)
            );
            step(
                &mut state,
                touches(&[touch(8, TouchPhase::Started, Some((100, 90)))]),
                &targets,
            );
            assert_eq!(
                step(
                    &mut state,
                    touches(&[touch(8, TouchPhase::Ended, Some((100, 90)))]),
                    &targets
                )
                .activation
                .unwrap()
                .id,
                id(1)
            );
            assert!(step(&mut state, confirm(), &targets).activation.is_some());
        }
    }
}

#[test]
fn pointer_exit_and_sparse_touch_frames_are_not_focus_loss() {
    use gotoo_pixel_engine::Input;
    let input = Input::default();
    let mut state = ResolvedState::default();
    step(&mut state, pointer(Some((100, 90)), true, false), &scene());
    assert!(state.synchronize_input(&input));
    assert!(step(&mut state, pointer(None, false, false), &scene()).active(id(1)));
    let mut state = ResolvedState::default();
    step(
        &mut state,
        touches(&[touch(1, TouchPhase::Started, Some((100, 90)))]),
        &scene(),
    );
    assert!(state.synchronize_input(&input));
    assert!(step(&mut state, UiInput::default(), &scene()).active(id(1)));
}
fn nav(n: UiNavInput) -> UiInput<'static> {
    UiInput {
        nav: n,
        ..Default::default()
    }
}
fn confirm() -> UiInput<'static> {
    nav(UiNavInput {
        confirm: true,
        ..Default::default()
    })
}
fn pointer(position: Option<(i32, i32)>, pressed: bool, released: bool) -> UiInput<'static> {
    UiInput {
        pointer: UiPointerInput {
            position,
            pressed,
            released,
        },
        ..Default::default()
    }
}
fn touch(id: u64, phase: TouchPhase, position: Option<(i32, i32)>) -> Touch {
    Touch {
        id,
        phase,
        position,
    }
}
fn touches(events: &[Touch]) -> UiInput<'_> {
    UiInput {
        touches: events,
        ..Default::default()
    }
}
fn step(
    state: &mut ResolvedState,
    input: UiInput<'_>,
    targets: &[ResolvedTarget],
) -> resolved::ResolvedOutput {
    resolved::update(state, input, targets).unwrap()
}

#[test]
fn spatial_four_directions_and_navigation_never_activates() {
    // Catches linear policy, missing directional navigation, and nav activating.
    for (direction, expected) in [
        (
            UiNavInput {
                up: true,
                ..Default::default()
            },
            2,
        ),
        (
            UiNavInput {
                right: true,
                ..Default::default()
            },
            3,
        ),
        (
            UiNavInput {
                down: true,
                ..Default::default()
            },
            4,
        ),
        (
            UiNavInput {
                left: true,
                ..Default::default()
            },
            5,
        ),
    ] {
        let mut state = ResolvedState::default();
        step(&mut state, UiInput::default(), &scene());
        let out = step(&mut state, nav(direction), &scene());
        assert_eq!(out.focused, Some(id(expected)));
        assert_eq!(out.activation, None);
    }
}

#[test]
fn stable_identity_survives_reorder_and_moving_geometry() {
    let mut targets = scene();
    let mut state = ResolvedState::default();
    step(&mut state, pointer(Some((110, 90)), true, false), &targets);
    targets.reverse();
    targets[4].rect.x = 270;
    let out = step(&mut state, pointer(Some((280, 90)), false, true), &targets);
    assert_eq!(out.focused, Some(id(1)));
    assert_eq!(
        out.activation,
        Some(Activation {
            id: id(1),
            action: ActionId::new("select")
        })
    );
}

#[test]
fn spatial_ties_use_resolved_order_and_primary_distance_wins() {
    let center = target(
        1,
        Rect {
            x: 0,
            y: 30,
            width: 10,
            height: 10,
        },
    );
    let upper = target(
        2,
        Rect {
            x: 30,
            y: 10,
            width: 10,
            height: 10,
        },
    );
    let lower = target(
        3,
        Rect {
            x: 30,
            y: 50,
            width: 10,
            height: 10,
        },
    );
    let aligned = target(
        4,
        Rect {
            x: 60,
            y: 30,
            width: 10,
            height: 10,
        },
    );
    for (targets, expected) in [
        ([center, upper, lower, aligned], 2),
        ([center, lower, upper, aligned], 3),
    ] {
        let mut state = ResolvedState::default();
        let out = step(
            &mut state,
            nav(UiNavInput {
                right: true,
                ..Default::default()
            }),
            &targets,
        );
        assert_eq!(out.focused, Some(id(expected)));
    }
}

#[test]
fn painting_and_hit_testing_share_half_open_rectangles() {
    let targets = scene();
    let mut fb = Framebuffer::new(320, 240);
    let color = Pixel::rgb(10, 20, 30);
    for t in &targets {
        fb.fill_rect(t.rect.x, t.rect.y, t.rect.width, t.rect.height, color);
    }
    let mut state = ResolvedState::default();
    for (point, expected) in [
        ((90, 80), Some(id(1))),
        ((129, 109), Some(id(1))),
        ((130, 109), None),
        ((89, 80), None),
    ] {
        let out = step(&mut state, pointer(Some(point), false, false), &targets);
        assert_eq!(out.hovered, expected);
        assert_eq!(
            fb.pixel(point.0, point.1) == Some(color),
            expected.is_some()
        );
    }
}

#[test]
fn invalid_frames_cancel_captures_and_require_neutral_reentry() {
    let normal = scene();
    let mut invalids = Vec::new();
    let mut invalid = normal;
    invalid[1].id = UiId::ROOT;
    invalids.push((invalid, ResolvedError::RootId { index: 1 }));
    let mut invalid = normal;
    invalid[1].id = id(1);
    invalids.push((invalid, ResolvedError::DuplicateId { id: id(1) }));
    for (width, height) in [(0, 20), (20, 0)] {
        let mut invalid = normal;
        invalid[1].rect.width = width;
        invalid[1].rect.height = height;
        invalids.push((invalid, ResolvedError::EmptyRect { id: id(2) }));
    }
    let mut invalid = normal;
    invalid[1].rect = normal[0].rect;
    invalid[1].enabled = false;
    invalids.push((
        invalid,
        ResolvedError::Overlap {
            first: id(1),
            second: id(2),
        },
    ));
    for (invalid, error) in invalids {
        let mut state = ResolvedState::default();
        let out = step(&mut state, pointer(Some((100, 90)), true, false), &normal);
        assert!(out.active(id(1)));
        assert_eq!(
            resolved::update(&mut state, confirm(), &invalid).unwrap_err(),
            error
        );
        assert_eq!(
            step(&mut state, pointer(Some((100, 90)), false, true), &normal).activation,
            None
        );
        assert_eq!(step(&mut state, confirm(), &normal).activation, None);
        step(&mut state, UiInput::default(), &normal);
        assert!(step(&mut state, confirm(), &normal).activation.is_some());
    }
    assert!(UiId::from_stable(UiId::ROOT.raw()).is_none());
}

#[test]
fn geometry_validation_is_safe_at_integer_extremes_and_edges_can_touch() {
    let targets = [
        target(
            1,
            Rect {
                x: i32::MIN,
                y: i32::MIN,
                width: u32::MAX,
                height: u32::MAX,
            },
        ),
        target(
            2,
            Rect {
                x: i32::MAX,
                y: 0,
                width: u32::MAX,
                height: 10,
            },
        ),
    ];
    let mut state = ResolvedState::default();
    assert_eq!(
        step(
            &mut state,
            pointer(Some((i32::MAX, 0)), true, true),
            &targets
        )
        .activation
        .unwrap()
        .id,
        id(2)
    );
    let mut overlap = targets;
    overlap[1].rect.x = i32::MAX - 1;
    assert!(matches!(
        resolved::update(&mut state, UiInput::default(), &overlap),
        Err(ResolvedError::Overlap { .. })
    ));
}

#[test]
fn disabled_and_removed_targets_repair_focus_without_confirming_fallback() {
    for remove in [false, true] {
        let normal = scene();
        let mut targets = normal.to_vec();
        let mut state = ResolvedState::default();
        step(&mut state, pointer(Some((100, 90)), true, false), &normal);
        if remove {
            targets.remove(0);
        } else {
            targets[0].enabled = false;
        }
        let out = step(&mut state, confirm(), &targets);
        assert_eq!(out.focused, Some(id(2)));
        assert!(!out.active(id(1)));
        assert_eq!(out.activation, None);
        assert_eq!(
            step(&mut state, pointer(Some((100, 90)), false, true), &normal).activation,
            None
        );
        assert_eq!(
            step(&mut state, UiInput::default(), &normal).focused,
            Some(id(2))
        );
    }
    let mut targets = scene();
    targets[0].enabled = false;
    let mut state = ResolvedState::default();
    let out = step(&mut state, pointer(Some((100, 90)), true, true), &targets);
    assert_eq!(out.hovered, None);
    assert_eq!(out.focused, Some(id(2)));
    assert_eq!(out.activation, None);
    assert_eq!(step(&mut state, UiInput::default(), &[]).focused, None);
}

#[test]
fn mouse_capture_release_inside_outside_and_missing_position() {
    for (position, activated) in [
        (Some((100, 90)), true),
        (Some((150, 90)), false),
        (None, false),
    ] {
        let mut state = ResolvedState::default();
        let out = step(&mut state, pointer(Some((100, 90)), true, false), &scene());
        assert!(out.active(id(1)));
        assert_eq!(out.activation, None);
        let out = step(&mut state, pointer(position, false, true), &scene());
        assert_eq!(out.activation.is_some(), activated);
        assert!(!out.active(id(1)));
        assert_eq!(
            step(&mut state, pointer(Some((100, 90)), false, true), &scene()).activation,
            None
        );
    }
}

#[test]
fn touch_move_out_and_cancel_discard_capture() {
    for phase in [TouchPhase::Moved, TouchPhase::Cancelled] {
        let mut state = ResolvedState::default();
        let out = step(
            &mut state,
            touches(&[touch(42, TouchPhase::Started, Some((100, 90)))]),
            &scene(),
        );
        assert!(out.active(id(1)));
        let out = step(
            &mut state,
            touches(&[touch(42, phase, Some((150, 90)))]),
            &scene(),
        );
        assert!(!out.active(id(1)));
        assert_eq!(
            step(
                &mut state,
                touches(&[touch(42, TouchPhase::Ended, Some((100, 90)))]),
                &scene()
            )
            .activation,
            None
        );
    }
    let mut state = ResolvedState::default();
    step(
        &mut state,
        touches(&[touch(42, TouchPhase::Started, Some((100, 90)))]),
        &scene(),
    );
    let out = step(
        &mut state,
        touches(&[touch(42, TouchPhase::Ended, Some((100, 90)))]),
        &scene(),
    );
    assert_eq!(out.activation.unwrap().id, id(1));
    assert!(!out.active(id(1)));
}

#[test]
fn cancel_dominates_confirm_navigation_and_gesture_end() {
    let mut state = ResolvedState::default();
    step(&mut state, pointer(Some((100, 90)), true, false), &scene());
    let mut input = pointer(Some((100, 90)), false, true);
    input.nav = UiNavInput {
        cancel: true,
        confirm: true,
        right: true,
        ..Default::default()
    };
    let out = step(&mut state, input, &scene());
    assert!(out.cancelled);
    assert_eq!(out.activation, None);
    assert_eq!(out.focused, Some(id(1)));
    assert!(!out.active(id(1)));
    assert_eq!(step(&mut state, confirm(), &scene()).activation, None);
}

#[test]
fn directions_and_ongoing_gestures_suppress_concurrent_confirm() {
    let mut state = ResolvedState::default();
    let out = step(
        &mut state,
        nav(UiNavInput {
            confirm: true,
            right: true,
            ..Default::default()
        }),
        &scene(),
    );
    assert_eq!(out.focused, Some(id(3)));
    assert_eq!(out.activation, None);
    step(&mut state, pointer(Some((100, 90)), true, false), &scene());
    let out = step(&mut state, confirm(), &scene());
    assert_eq!(out.activation, None);
    assert!(out.active(id(1)));
    let mut state = ResolvedState::default();
    step(
        &mut state,
        touches(&[touch(7, TouchPhase::Started, Some((100, 90)))]),
        &scene(),
    );
    assert_eq!(step(&mut state, confirm(), &scene()).activation, None);
    let out = step(
        &mut state,
        nav(UiNavInput {
            left: true,
            right: true,
            confirm: true,
            ..Default::default()
        }),
        &[],
    );
    assert_eq!(out.activation, None);
}

#[test]
fn confirm_with_click_has_one_pointer_activation_even_on_another_target() {
    for (point, expected) in [((100, 90), 1), ((200, 90), 3)] {
        let mut state = ResolvedState::default();
        step(&mut state, UiInput::default(), &scene());
        let mut input = pointer(Some(point), true, true);
        input.nav.confirm = true;
        let out = step(&mut state, input, &scene());
        assert_eq!(out.activation.unwrap().id, id(expected));
    }
}

#[test]
fn multiple_pointer_ids_are_ambiguous_but_same_id_is_deduplicated() {
    for (second, expected) in [((200, 90), None), ((100, 90), Some(id(1)))] {
        let mut state = ResolvedState::default();
        step(
            &mut state,
            touches(&[
                touch(10, TouchPhase::Started, Some((100, 90))),
                touch(11, TouchPhase::Started, Some(second)),
            ]),
            &scene(),
        );
        let out = step(
            &mut state,
            touches(&[
                touch(11, TouchPhase::Ended, Some(second)),
                touch(10, TouchPhase::Ended, Some((100, 90))),
            ]),
            &scene(),
        );
        assert_eq!(out.activation.map(|a| a.id), expected);
    }
}

#[test]
fn suspension_neutralizes_next_frame_and_prevents_delayed_activation() {
    let mut state = ResolvedState::default();
    step(&mut state, pointer(Some((100, 90)), true, false), &scene());
    step(
        &mut state,
        touches(&[touch(77, TouchPhase::Started, Some((200, 90)))]),
        &scene(),
    );
    state.suspend();
    let out = step(&mut state, confirm(), &scene());
    assert_eq!(out.focused, Some(id(3)));
    assert!(!out.active(id(1)));
    assert!(!out.active(id(3)));
    assert_eq!(out.activation, None);
    assert_eq!(
        step(&mut state, pointer(Some((100, 90)), false, true), &scene()).activation,
        None
    );
    assert_eq!(
        step(
            &mut state,
            touches(&[touch(77, TouchPhase::Ended, Some((200, 90)))]),
            &scene()
        )
        .activation,
        None
    );
    step(&mut state, UiInput::default(), &scene());
    assert_eq!(
        step(&mut state, confirm(), &scene()).activation.unwrap().id,
        id(3)
    );
    state.suspend();
    step(&mut state, UiInput::default(), &scene());
    assert_eq!(
        step(&mut state, pointer(Some((100, 90)), false, true), &scene()).activation,
        None
    );
}

#[test]
fn disabling_or_removing_a_touch_capture_cannot_resume_it_on_reinsert() {
    for remove in [false, true] {
        let mut state = ResolvedState::default();
        step(
            &mut state,
            touches(&[touch(9, TouchPhase::Started, Some((100, 90)))]),
            &scene(),
        );
        let mut targets = scene().to_vec();
        if remove {
            targets.remove(0);
        } else {
            targets[0].enabled = false;
        }
        let out = step(&mut state, UiInput::default(), &targets);
        assert!(!out.active(id(1)));
        assert_eq!(
            step(
                &mut state,
                touches(&[touch(9, TouchPhase::Ended, Some((100, 90)))]),
                &scene()
            )
            .activation,
            None
        );
    }
}

#[test]
fn mixed_mouse_and_touch_activations_use_id_ambiguity_not_event_order() {
    for (point, expected) in [((100, 90), Some(id(1))), ((200, 90), None)] {
        let mut state = ResolvedState::default();
        let start = [touch(20, TouchPhase::Started, Some(point))];
        let mut input = pointer(Some((100, 90)), true, false);
        input.touches = &start;
        step(&mut state, input, &scene());
        let end = [touch(20, TouchPhase::Ended, Some(point))];
        let mut input = pointer(Some((100, 90)), false, true);
        input.touches = &end;
        input.nav.confirm = true;
        let out = step(&mut state, input, &scene());
        assert_eq!(out.activation.map(|a| a.id), expected);
    }
}

#[test]
fn spatial_secondary_distance_and_previous_index_fallback_stay_in_kernel() {
    let mut targets = scene();
    // Same primary distance to the right, different secondary distance.
    targets[1].rect = Rect {
        x: 180,
        y: 10,
        width: 60,
        height: 20,
    };
    let mut state = ResolvedState::default();
    let out = step(
        &mut state,
        nav(UiNavInput {
            right: true,
            ..Default::default()
        }),
        &targets,
    );
    assert_eq!(out.focused, Some(id(3)));
    targets[2].enabled = false;
    let out = step(&mut state, confirm(), &targets);
    // Previous enabled index 2 now points to the fourth control, not the first.
    assert_eq!(out.focused, Some(id(4)));
    assert_eq!(out.activation, None);
    for target in &mut targets {
        target.enabled = false;
    }
    let out = step(&mut state, confirm(), &targets);
    assert_eq!(out.focused, None);
    assert_eq!(out.activation, None);
}

#[test]
fn touch_moving_inside_keeps_capture_and_release_outside_does_not_activate() {
    let mut state = ResolvedState::default();
    step(
        &mut state,
        touches(&[touch(5, TouchPhase::Started, Some((100, 90)))]),
        &scene(),
    );
    let out = step(
        &mut state,
        touches(&[touch(5, TouchPhase::Moved, Some((120, 100)))]),
        &scene(),
    );
    assert!(out.active(id(1)));
    assert_eq!(out.activation, None);
    let out = step(
        &mut state,
        touches(&[touch(5, TouchPhase::Ended, Some((200, 90)))]),
        &scene(),
    );
    assert!(!out.active(id(1)));
    assert_eq!(out.activation, None);
}

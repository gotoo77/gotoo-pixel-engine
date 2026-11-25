use super::*;
use crate::Key;

const LEFT: ActionId = ActionId::new("test.left");
const RIGHT: ActionId = ActionId::new("test.right");
const MENU: ActionId = ActionId::new("test.menu");
const PLAY: ControlContext = ControlContext("play");
const PAUSE: ControlContext = ControlContext("pause");

fn slot(default: ControlBinding) -> BindingSlot {
    BindingSlot {
        default,
        editable: true,
    }
}

fn action(id: ActionId, context: ControlContext, slots: Vec<BindingSlot>) -> ControlAction {
    ControlAction {
        id,
        label: id.as_str(),
        context,
        slots,
    }
}

fn config() -> ControlConfiguration {
    ControlConfiguration::new(
        vec![
            action(
                LEFT,
                PLAY,
                vec![
                    slot(ControlBinding::Key(Key::Left)),
                    slot(ControlBinding::Gamepad(GamepadButton::DPadLeft)),
                ],
            ),
            action(RIGHT, PLAY, vec![slot(ControlBinding::Key(Key::Right))]),
        ],
        vec![],
        vec![ControlBinding::Key(Key::Escape)],
    )
    .unwrap()
}

#[test]
fn replacement_preserves_other_sources_and_restore() {
    let mut config = config();
    config
        .replace(&[(LEFT, 0, ControlBinding::Key(Key::A))])
        .unwrap();
    assert_eq!(
        config.bindings(LEFT).unwrap(),
        [
            ControlBinding::Key(Key::A),
            ControlBinding::Gamepad(GamepadButton::DPadLeft),
        ]
    );
    assert_eq!(config.resolve_action("test.left"), Some(LEFT));
    assert_eq!(config.resolve_action("missing"), None);
    config.restore_defaults();
    assert_eq!(
        config.bindings(LEFT).unwrap()[0],
        ControlBinding::Key(Key::Left)
    );
}

#[test]
fn rejected_batch_preserves_all_bindings() {
    let mut config = config();
    let before = config.active.clone();
    assert!(matches!(
        config.replace(&[
            (LEFT, 0, ControlBinding::Key(Key::A)),
            (RIGHT, 0, ControlBinding::Key(Key::A)),
        ]),
        Err(ConfigurationError::Conflict(LEFT, RIGHT, _, _))
    ));
    assert_eq!(config.active, before);
    assert!(matches!(
        config.replace(&[
            (LEFT, 0, ControlBinding::Key(Key::A)),
            (RIGHT, 9, ControlBinding::Key(Key::D)),
        ]),
        Err(ConfigurationError::UnknownSlot(RIGHT, 9))
    ));
    assert_eq!(config.active, before);
    assert!(matches!(
        config.replace(&[(LEFT, 0, ControlBinding::Key(Key::Escape))]),
        Err(ConfigurationError::Reserved(_))
    ));
    assert_eq!(config.active, before);
    assert!(matches!(
        config.replace(&[(LEFT, 0, ControlBinding::Gamepad(GamepadButton::South))]),
        Err(ConfigurationError::WrongSource(LEFT, 0))
    ));
    assert_eq!(config.active, before);
    assert!(matches!(
        config.replace(&[(MENU, 0, ControlBinding::Key(Key::A))]),
        Err(ConfigurationError::UnknownAction(MENU))
    ));
    assert_eq!(config.active, before);
}

#[test]
fn batch_validates_final_configuration() {
    let mut config = config();
    config
        .replace(&[
            (LEFT, 0, ControlBinding::Key(Key::Right)),
            (RIGHT, 0, ControlBinding::Key(Key::Left)),
        ])
        .unwrap();
    assert_eq!(
        config.bindings(LEFT).unwrap()[0],
        ControlBinding::Key(Key::Right)
    );
}

#[test]
fn contexts_overlap_unless_explicitly_exclusive() {
    let actions = vec![
        action(LEFT, PLAY, vec![slot(ControlBinding::Key(Key::A))]),
        action(MENU, PAUSE, vec![slot(ControlBinding::Key(Key::A))]),
    ];
    assert!(matches!(
        ControlConfiguration::new(actions.clone(), vec![], vec![]),
        Err(ConfigurationError::Conflict(LEFT, MENU, _, _))
    ));
    assert!(ControlConfiguration::new(actions.clone(), vec![(PAUSE, PLAY)], vec![]).is_ok());
    assert!(matches!(
        ControlConfiguration::new(actions, vec![(PLAY, PLAY)], vec![]),
        Err(ConfigurationError::InvalidExclusivity(_, _))
    ));
}

#[test]
fn any_device_overlaps_specific_but_distinct_devices_do_not() {
    let button = GamepadButton::South;
    let first = ControlBinding::GamepadDevice(GamepadId::new(1), button);
    let second = ControlBinding::GamepadDevice(GamepadId::new(2), button);
    let actions = vec![
        action(LEFT, PLAY, vec![slot(first)]),
        action(RIGHT, PLAY, vec![slot(second)]),
    ];
    let mut config = ControlConfiguration::new(actions, vec![], vec![]).unwrap();
    assert!(matches!(
        config.replace(&[(LEFT, 0, second)]),
        Err(ConfigurationError::WrongSource(LEFT, 0))
    ));
    assert!(matches!(
        ControlConfiguration::new(
            vec![
                action(LEFT, PLAY, vec![slot(first)]),
                action(RIGHT, PLAY, vec![slot(ControlBinding::Gamepad(button))]),
            ],
            vec![],
            vec![],
        ),
        Err(ConfigurationError::Conflict(LEFT, RIGHT, _, _))
    ));
}

#[test]
fn fixed_reserved_slots_and_duplicate_actions() {
    let fixed = BindingSlot {
        default: ControlBinding::Key(Key::Escape),
        editable: false,
    };
    let entry = action(MENU, PAUSE, vec![fixed]);
    let mut config = ControlConfiguration::new(
        vec![entry.clone()],
        vec![],
        vec![fixed.default],
    )
    .unwrap();
    assert!(matches!(
        config.replace(&[(MENU, 0, ControlBinding::Key(Key::A))]),
        Err(ConfigurationError::FixedSlot(MENU, 0))
    ));
    assert!(matches!(
        ControlConfiguration::new(vec![entry.clone(), entry], vec![], vec![]),
        Err(ConfigurationError::DuplicateAction(MENU))
    ));
}

#[test]
fn apply_preserves_unmanaged_actions_and_virtual_sources() {
    let mut config = config();
    let mut map = ControlMap::new();
    map.bind_key(MENU, Key::M);
    map.set_virtual(LEFT, true);
    config.apply_to(&mut map);
    let mut input = crate::Input::default();
    input.press_key(Key::M);
    map.update(&input);
    assert!(map.action(MENU).pressed());
    assert!(map.action(LEFT).pressed());
    config
        .replace(&[(LEFT, 0, ControlBinding::Key(Key::A))])
        .unwrap();
    config.apply_to(&mut map);
    map.update(&input);
    assert!(map.action(LEFT).held());
    assert!(!map.action(LEFT).pressed());
    map.set_virtual(LEFT, false);
    input.press_key(Key::Left);
    map.update(&input);
    assert!(map.action(LEFT).released());
    input.press_key(Key::A);
    map.update(&input);
    assert!(map.action(LEFT).pressed());
    assert!(map.action(MENU).held());
}

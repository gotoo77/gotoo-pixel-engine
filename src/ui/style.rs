use crate::Pixel;

use super::UiTheme;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiStyleOverride {
    pub background: Option<Pixel>,
    pub border: Option<Pixel>,
    pub text: Option<Pixel>,
    pub muted_text: Option<Pixel>,
    pub accent: Option<Pixel>,
    pub border_width: Option<u32>,
    pub padding: Option<u32>,
    pub vertical_gap: Option<u32>,
}

impl UiStyleOverride {
    fn apply_to(self, style: &mut UiResolvedStyle) {
        if let Some(value) = self.background {
            style.background = value;
        }
        if let Some(value) = self.border {
            style.border = value;
        }
        if let Some(value) = self.text {
            style.text = value;
        }
        if let Some(value) = self.muted_text {
            style.muted_text = value;
        }
        if let Some(value) = self.accent {
            style.accent = value;
        }
        if let Some(value) = self.border_width {
            style.border_width = value;
        }
        if let Some(value) = self.padding {
            style.padding = value;
        }
        if let Some(value) = self.vertical_gap {
            style.vertical_gap = value;
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiComponentStyle {
    pub base: UiStyleOverride,
    pub focused: UiStyleOverride,
    pub hovered: UiStyleOverride,
    pub active: UiStyleOverride,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiStyleSheet {
    pub panel: UiComponentStyle,
    pub text: UiComponentStyle,
    pub button: UiComponentStyle,
    pub toggle_bool: UiComponentStyle,
    pub slider_f32: UiComponentStyle,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiVisualState {
    pub focused: bool,
    pub hovered: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UiResolvedStyle {
    pub background: Pixel,
    pub border: Pixel,
    pub text: Pixel,
    pub muted_text: Pixel,
    pub accent: Pixel,
    pub border_width: u32,
    pub padding: u32,
    pub vertical_gap: u32,
}

impl UiResolvedStyle {
    pub(crate) fn from_theme(theme: UiTheme) -> Self {
        Self {
            background: theme.control_background,
            border: theme.border,
            text: theme.text,
            muted_text: theme.muted_text,
            accent: theme.accent,
            border_width: 1,
            padding: theme.padding,
            vertical_gap: theme.row_spacing,
        }
    }
}

pub(crate) fn resolve_style(
    theme: UiTheme,
    component: UiComponentStyle,
    local: UiStyleOverride,
    visual: UiVisualState,
) -> UiResolvedStyle {
    let mut resolved = UiResolvedStyle::from_theme(theme);
    component.base.apply_to(&mut resolved);
    local.apply_to(&mut resolved);

    if visual.focused {
        component.focused.apply_to(&mut resolved);
    }
    if visual.hovered {
        component.hovered.apply_to(&mut resolved);
    }
    if visual.active {
        component.active.apply_to(&mut resolved);
    }

    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(value: u8) -> Pixel {
        Pixel::rgb(value, value, value)
    }

    #[test]
    fn compatibility_style_is_derived_from_unchanged_theme() {
        let theme = UiTheme {
            control_background: color(1),
            border: color(2),
            text: color(3),
            muted_text: color(4),
            accent: color(5),
            padding: 6,
            row_spacing: 7,
            ..UiTheme::default()
        };

        let resolved = resolve_style(
            theme,
            UiComponentStyle::default(),
            UiStyleOverride::default(),
            UiVisualState::default(),
        );

        assert_eq!(resolved.background, color(1));
        assert_eq!(resolved.border, color(2));
        assert_eq!(resolved.text, color(3));
        assert_eq!(resolved.muted_text, color(4));
        assert_eq!(resolved.accent, color(5));
        assert_eq!(resolved.border_width, 1);
        assert_eq!(resolved.padding, 6);
        assert_eq!(resolved.vertical_gap, 7);
    }

    #[test]
    fn static_precedence_is_theme_then_component_then_local_field_by_field() {
        let theme = UiTheme {
            control_background: color(1),
            border: color(2),
            text: color(3),
            padding: 4,
            ..UiTheme::default()
        };
        let component = UiComponentStyle {
            base: UiStyleOverride {
                background: Some(color(10)),
                border: Some(color(11)),
                padding: Some(12),
                ..UiStyleOverride::default()
            },
            ..UiComponentStyle::default()
        };
        let local = UiStyleOverride {
            border: Some(color(20)),
            text: Some(color(21)),
            ..UiStyleOverride::default()
        };

        let resolved = resolve_style(theme, component, local, UiVisualState::default());

        assert_eq!(resolved.background, color(10));
        assert_eq!(resolved.border, color(20));
        assert_eq!(resolved.text, color(21));
        assert_eq!(resolved.padding, 12);
    }

    #[test]
    fn visual_precedence_is_active_over_hovered_over_focused_over_static() {
        let component = UiComponentStyle {
            base: UiStyleOverride {
                border: Some(color(10)),
                ..UiStyleOverride::default()
            },
            focused: UiStyleOverride {
                border: Some(color(20)),
                text: Some(color(21)),
                ..UiStyleOverride::default()
            },
            hovered: UiStyleOverride {
                border: Some(color(30)),
                background: Some(color(31)),
                ..UiStyleOverride::default()
            },
            active: UiStyleOverride {
                border: Some(color(40)),
                ..UiStyleOverride::default()
            },
        };
        let local = UiStyleOverride {
            border: Some(color(15)),
            accent: Some(color(16)),
            ..UiStyleOverride::default()
        };

        let resolved = resolve_style(
            UiTheme::default(),
            component,
            local,
            UiVisualState {
                focused: true,
                hovered: true,
                active: true,
            },
        );

        assert_eq!(resolved.border, color(40));
        assert_eq!(resolved.background, color(31));
        assert_eq!(resolved.text, color(21));
        assert_eq!(resolved.accent, color(16));
    }

    #[test]
    fn inactive_visual_layers_do_not_override_static_style() {
        let component = UiComponentStyle {
            base: UiStyleOverride {
                border: Some(color(10)),
                ..UiStyleOverride::default()
            },
            focused: UiStyleOverride {
                border: Some(color(20)),
                ..UiStyleOverride::default()
            },
            hovered: UiStyleOverride {
                border: Some(color(30)),
                ..UiStyleOverride::default()
            },
            active: UiStyleOverride {
                border: Some(color(40)),
                ..UiStyleOverride::default()
            },
        };

        let resolved = resolve_style(
            UiTheme::default(),
            component,
            UiStyleOverride::default(),
            UiVisualState::default(),
        );

        assert_eq!(resolved.border, color(10));
    }
}

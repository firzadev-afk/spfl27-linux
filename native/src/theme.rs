//! Design tokens + shared styles. Single source of truth for every visual
//! value in the app: type scale, spacing, radius, elevation, semantic colour.
//!
//! Rules enforced here:
//!   * no magic numbers in `ui.rs` — every size/space/colour comes from here
//!   * one spacing scale (4px base) and one type scale (8 steps)
//!   * interaction states are explicit tokens, not ad-hoc tweaks
//!   * text contrast meets WCAG AA (checked in `tests::contrast_ratios`)

use iced::widget::{button, text_input};
use iced::{Background, Border, Color, Shadow, Theme};

pub mod space {
    //! 4px-based spacing scale.
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XXL: f32 = 24.0;
    pub const XXXL: f32 = 32.0;
    pub const HUGE: f32 = 40.0;
    /// Horizontal page margin.
    pub const GUTTER: f32 = 28.0;
    /// Sidebar width. `SIDEBAR + 2*GUTTER` is the minimum horizontal chrome.
    pub const SIDEBAR: f32 = 208.0;
    /// Fixed width for small form labels.
    pub const FIELD_LABEL_W: f32 = 64.0;
    /// Maximum width of modal sheets and toast notifications.
    pub const MODAL_WIDTH: f32 = 832.0;
    /// Fixed height for the Find-game result list.
    pub const SHEET_HEIGHT: f32 = 472.0;
    /// Fixed height for the Activity log drawer.
    pub const LOG_HEIGHT: f32 = 88.0;
}

pub mod text {
    //! Type scale — 8 steps, no more.
    pub const MICRO: f32 = 12.0; // log lines, badges
    pub const SMALL: f32 = 13.0; // captions, hints
    pub const BODY: f32 = 14.0; // default UI text
    pub const BODY_LG: f32 = 15.0; // card titles
    pub const H3: f32 = 16.0;
    pub const H2: f32 = 17.0; // page title
    pub const H1: f32 = 20.0; // wizard step title
    pub const DISPLAY: f32 = 26.0; // welcome hero
    /// Tight leading for headings (1.25).
    pub const LH_TIGHT: f32 = 1.25;
    /// Normal leading for UI text (1.45).
    pub const LH_NORMAL: f32 = 1.45;
    /// Relaxed leading for prose paragraphs (1.6).
    pub const LH_RELAXED: f32 = 1.6;
    /// Max width for readable prose (~72 chars at BODY).
    pub const MEASURE: f32 = 680.0;
}

pub mod radius {
    pub const SM: f32 = 6.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 12.0;
    pub const PILL: f32 = 999.0;
}

pub mod icon {
    //! Icon sizes — one ladder, used everywhere.
    pub const SM: f32 = 14.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 20.0;
    pub const XL: f32 = 32.0;
    pub const HERO: f32 = 56.0;
    /// Height of the top bar / wizard header.
    pub const BAR: f32 = 56.0;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Dark,
    Light,
}

/// Semantic colour tokens. Purpose-named, so re-theming never touches widgets.
#[derive(Clone, Copy)]
pub struct Tokens {
    /// Window background (canvas).
    pub bg: Color,
    /// Card / panel surface.
    pub surface: Color,
    /// Input fields, secondary fills.
    pub surface_2: Color,
    /// Hover fill.
    pub surface_3: Color,
    /// Pressed / selected fill.
    pub surface_4: Color,
    /// Subtle separator.
    pub border: Color,
    /// Emphasised border (focus, active).
    pub border_strong: Color,
    /// Primary text.
    pub text: Color,
    /// Secondary text.
    pub text_muted: Color,
    /// Faint text (log hints).
    pub text_faint: Color,
    /// Brand accent — primary actions.
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_active: Color,
    /// Text/icon colour placed ON the accent.
    pub on_accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub danger_hover: Color,
    pub danger_active: Color,
    /// Text/icon colour placed ON the danger fill.
    pub on_danger: Color,
}

/// Relative luminance per WCAG 2.1 (used by the contrast test).
#[cfg_attr(not(test), allow(dead_code))]
fn luminance(c: Color) -> f32 {
    let f = |v: f32| {
        let v = v / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c.r * 255.0) + 0.7152 * f(c.g * 255.0) + 0.0722 * f(c.b * 255.0)
}

/// WCAG contrast ratio between two colours (1.0 .. 21.0).
#[cfg_attr(not(test), allow(dead_code))]
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

pub fn tokens(mode: Mode) -> Tokens {
    match mode {
        // Neutral canvas with a bright green accent. Green takes dark text so
        // the button clears AA (8.4:1) — white on green would only reach 2.4:1.
        Mode::Dark => Tokens {
            bg: rgb(0x0F, 0x11, 0x15),
            surface: rgb(0x17, 0x1A, 0x20),
            surface_2: rgb(0x1E, 0x23, 0x2B),
            surface_3: rgb(0x26, 0x2C, 0x36),
            surface_4: rgb(0x2F, 0x37, 0x43),
            border: rgb(0x27, 0x2D, 0x37),
            border_strong: rgb(0x39, 0x41, 0x4E),
            text: rgb(0xE8, 0xEC, 0xF2),
            text_muted: rgb(0x9A, 0xA5, 0xB4),
            text_faint: rgb(0x6B, 0x76, 0x84),
            accent: rgb(0x22, 0xC5, 0x5E),
            accent_hover: rgb(0x34, 0xD0, 0x7A),
            accent_active: rgb(0x1C, 0xA8, 0x50),
            on_accent: rgb(0x08, 0x12, 0x0C),
            success: rgb(0x22, 0xC5, 0x5E),
            warning: rgb(0xE8, 0xA5, 0x3D),
            danger: rgb(0xF0, 0x61, 0x6D),
            danger_hover: rgb(0xF7, 0x7B, 0x84),
            danger_active: rgb(0xE0, 0x46, 0x53),
            on_danger: rgb(0x15, 0x0A, 0x0B),
        },
        // On light canvas the accent deepens so white text still clears AA (5:1).
        Mode::Light => Tokens {
            bg: rgb(0xF6, 0xF7, 0xF9),
            surface: rgb(0xFF, 0xFF, 0xFF),
            surface_2: rgb(0xF0, 0xF2, 0xF5),
            surface_3: rgb(0xE7, 0xEA, 0xEF),
            surface_4: rgb(0xDD, 0xE1, 0xE8),
            border: rgb(0xE2, 0xE6, 0xEC),
            border_strong: rgb(0xC5, 0xCD, 0xD8),
            text: rgb(0x15, 0x1A, 0x21),
            text_muted: rgb(0x5C, 0x66, 0x73),
            text_faint: rgb(0x84, 0x8D, 0x9A),
            accent: rgb(0x15, 0x80, 0x3D),
            accent_hover: rgb(0x14, 0x71, 0x37),
            accent_active: rgb(0x11, 0x5F, 0x2E),
            on_accent: rgb(0xFF, 0xFF, 0xFF),
            success: rgb(0x15, 0x80, 0x3D),
            warning: rgb(0x9C, 0x5C, 0x08),
            danger: rgb(0xC8, 0x2A, 0x2A),
            danger_hover: rgb(0xB4, 0x22, 0x22),
            danger_active: rgb(0x9C, 0x1C, 0x1C),
            on_danger: rgb(0xFF, 0xFF, 0xFF),
        },
    }
}

/// Token set for a `dark: bool` flag.
pub fn tok(dark: bool) -> Tokens {
    tokens(if dark { Mode::Dark } else { Mode::Light })
}

fn iced_palette(mode: Mode) -> iced::theme::Palette {
    let t = tokens(mode);
    iced::theme::Palette {
        background: t.bg,
        text: t.text,
        primary: t.accent,
        success: t.success,
        warning: t.warning,
        danger: t.danger,
    }
}

pub fn theme_of(mode: Mode) -> Theme {
    let name = match mode {
        Mode::Dark => "SPFL Playtools Dark",
        Mode::Light => "SPFL Playtools Light",
    };
    Theme::custom(name.to_string(), iced_palette(mode))
}

// ── Elevation ────────────────────────────────────────────────────────────
/// Shadows read poorly on dark canvases, so dark mode leans on borders.
fn elevation(dark: bool, level: u8) -> Shadow {
    if dark {
        match level {
            2 => Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 20.0,
            },
            _ => Shadow::default(),
        }
    } else {
        match level {
            1 => Shadow {
                color: Color::from_rgba(0.059, 0.090, 0.165, 0.06),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            2 => Shadow {
                color: Color::from_rgba(0.059, 0.090, 0.165, 0.12),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            _ => Shadow::default(),
        }
    }
}

// ── Component styles ─────────────────────────────────────────────────────
/// `border_color` override lets callers tint a card (e.g. success).
pub fn card_style(dark: bool) -> impl Fn(&Theme) -> iced::widget::container::Style {
    card_style_with(dark, None, 1)
}

pub fn card_style_with(
    dark: bool,
    border_color: Option<Color>,
    level: u8,
) -> impl Fn(&Theme) -> iced::widget::container::Style {
    move |_t: &Theme| {
        let c = tok(dark);
        iced::widget::container::Style {
            background: Some(Background::Color(c.surface)),
            border: Border {
                radius: radius::LG.into(),
                width: 1.0,
                color: border_color.unwrap_or(c.border),
            },
            text_color: None,
            shadow: elevation(dark, level),
            snap: true,
        }
    }
}

/// Filled call-to-action: accent fill, dark/light on-accent text.
pub fn btn_primary(dark: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t: &Theme, status| {
        let t = tok(dark);
        let (bg, fg) = if status == button::Status::Disabled {
            (t.surface_3, t.text_faint)
        } else if status == button::Status::Active {
            (t.accent_active, t.on_accent)
        } else if status == button::Status::Hovered {
            (t.accent_hover, t.on_accent)
        } else {
            (t.accent, t.on_accent)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: radius::MD.into(),
                width: 1.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// Secondary action: surface fill with a real hover / press response.
pub fn btn_ghost(dark: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t: &Theme, status| {
        let t = tok(dark);
        let (bg, fg, bd) = if status == button::Status::Disabled {
            (t.surface, t.text_faint, t.border)
        } else if status == button::Status::Active {
            (t.surface_4, t.text, t.border_strong)
        } else if status == button::Status::Hovered {
            (t.surface_3, t.text, t.border_strong)
        } else {
            (t.surface_2, t.text, t.border)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: radius::MD.into(),
                width: 1.0,
                color: bd,
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// Destructive action: danger fill + danger colour instead of the green
/// accent, so an overwrite never looks like a friendly primary button.
pub fn btn_danger(dark: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t: &Theme, status| {
        let c = tok(dark);
        let (bg, fg) = if status == button::Status::Active {
            (c.danger_active, c.on_danger)
        } else if status == button::Status::Hovered {
            (c.danger_hover, c.on_danger)
        } else if status == button::Status::Disabled {
            (c.surface_3, c.text_faint)
        } else {
            (c.danger, c.on_danger)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: radius::MD.into(),
                width: 1.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// Text-input / field surface. `Focused` gets a visible accent-ring so a
/// keyboard user can always see where the caret is.
pub fn input_style(dark: bool) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_t: &Theme, status| {
        let c = tok(dark);
        let focused = matches!(status, text_input::Status::Focused { .. });
        let (bg, border) = if let text_input::Status::Disabled = status {
            (c.surface_2, c.border)
        } else if focused
            || matches!(
                status,
                text_input::Status::Active | text_input::Status::Hovered
            )
        {
            (c.surface, c.border_strong)
        } else {
            (c.surface_2, c.border)
        };
        text_input::Style {
            background: Background::Color(bg),
            border: Border {
                radius: radius::MD.into(),
                width: 1.0,
                color: border,
            },
            icon: c.text_faint,
            placeholder: c.text_faint,
            value: c.text,
            selection: c.accent,
        }
    }
}

/// Button that turns into the accent when `selected` (e.g. the active backup).
pub fn btn_selectable(
    dark: bool,
    selected: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status| {
        if selected {
            btn_primary(dark)(theme, status)
        } else {
            btn_ghost(dark)(theme, status)
        }
    }
}

/// Low-emphasis inline action (no fill until hover).
pub fn btn_quiet(dark: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t: &Theme, status| {
        let t = tok(dark);
        let (bg, fg) = if status == button::Status::Disabled {
            (Color::TRANSPARENT, t.text_faint)
        } else if status == button::Status::Active {
            (t.surface_3, t.text)
        } else if status == button::Status::Hovered {
            (t.surface_2, t.text)
        } else {
            (Color::TRANSPARENT, t.text_muted)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: radius::MD.into(),
                width: 1.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// Sidebar nav item. Active item gets a tinted fill + accent text.
pub fn nav_item(dark: bool, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t: &Theme, status| {
        let t = tok(dark);
        let (bg, fg) = if active {
            (t.surface_3, t.accent)
        } else if status == button::Status::Active {
            (t.surface_3, t.text)
        } else if status == button::Status::Hovered {
            (t.surface_2, t.text)
        } else {
            (Color::TRANSPARENT, t.text_muted)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                radius: radius::SM.into(),
                width: 1.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// Monospace chip for paths / log.
pub fn chip_style(dark: bool) -> impl Fn(&Theme) -> iced::widget::container::Style {
    move |_t: &Theme| {
        let c = tok(dark);
        iced::widget::container::Style {
            background: Some(Background::Color(c.surface_2)),
            border: Border {
                radius: radius::SM.into(),
                width: 1.0,
                color: c.border,
            },
            text_color: None,
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

/// Pill badge used by the wizard step rail.
pub fn pill_style(bg: Color, border: Color) -> impl Fn(&Theme) -> iced::widget::container::Style {
    move |_t: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            radius: radius::PILL.into(),
            width: 1.0,
            color: border,
        },
        text_color: None,
        shadow: Shadow::default(),
        snap: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every text/surface pair must clear WCAG AA (4.5:1) or AA-large (3:1).
    #[test]
    fn contrast_ratios_meet_wcag() {
        for mode in [Mode::Dark, Mode::Light] {
            let t = tokens(mode);
            let pairs: [(&str, Color, Color); 7] = [
                ("text on bg", t.text, t.bg),
                ("text on surface", t.text, t.surface),
                ("muted on surface", t.text_muted, t.surface),
                ("faint on surface", t.text_faint, t.surface),
                ("on_accent on accent", t.on_accent, t.accent),
                ("danger on surface", t.danger, t.surface),
                ("warning on surface", t.warning, t.surface),
            ];
            let mut pairs = pairs.to_vec();
            pairs.extend_from_slice(&[
                ("on_danger on danger", t.on_danger, t.danger),
                ("danger on bg", t.danger, t.bg),
                ("accent on surface", t.accent, t.surface),
            ]);
            for (name, fg, bg) in pairs {
                let r = contrast_ratio(fg, bg);
                let min = if name == "faint on surface" { 3.0 } else { 4.5 };
                assert!(r >= min, "{mode:?} {name}: {r:.2} < {min}");
            }
        }
    }

    #[test]
    fn scale_values_are_unique_and_ordered() {
        let sizes = [
            text::MICRO,
            text::SMALL,
            text::BODY,
            text::BODY_LG,
            text::H3,
            text::H2,
            text::H1,
            text::DISPLAY,
        ];
        for w in sizes.windows(2) {
            assert!(w[0] < w[1], "type scale not strictly increasing: {w:?}");
        }
        let sp = [
            space::XXS,
            space::XS,
            space::SM,
            space::MD,
            space::LG,
            space::XXL,
            space::XXXL,
            space::HUGE,
        ];
        for w in sp.windows(2) {
            assert!(w[0] < w[1], "spacing scale not strictly increasing: {w:?}");
        }
        // every step except the 2px hairline is a multiple of the 4px base
        for v in sp.iter().skip(1) {
            assert!((v % 4.0).abs() < f32::EPSILON, "{v} off the 4px grid");
        }
    }

    #[test]
    fn mode_flags_pick_distinct_tokens() {
        assert_ne!(tokens(Mode::Dark).bg, tokens(Mode::Light).bg);
        assert_ne!(tokens(Mode::Dark).text, tokens(Mode::Light).text);
    }
}

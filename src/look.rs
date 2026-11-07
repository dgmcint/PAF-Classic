//! Colour schemes. Classic is the look of the original; the others follow the tree pictures.
use egui::Color32;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Every colour the screens use, by the job it does.
pub struct Palette {
    /// Window and panel background.
    pub bg: Color32,
    /// Slightly set-off background, such as striped rows.
    pub faint: Color32,
    /// Inside typing boxes.
    pub text_bg: Color32,
    /// Text and lines drawn on the background.
    pub ink: Color32,
    /// The white boxes that hold names and dates, and the text inside them.
    pub field: Color32,
    pub field_ink: Color32,
    /// The highlighted box or row, and the text on it.
    pub select: Color32,
    pub select_ink: Color32,
    /// The dark and light edges of raised and sunken parts.
    pub shadow: Color32,
    pub light: Color32,
    /// Buttons at rest, under the pointer and pressed.
    pub button: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    /// Pop-up hints.
    pub tip: Color32,
    /// Warnings such as a date that isn't standard.
    pub warn: Color32,
    /// Whether the background is dark, so the toolbar and menus use light text.
    pub dark: bool,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// `a` moved `pct` percent of the way to `b`.
const fn mix(a: Color32, b: Color32, pct: u32) -> Color32 {
    const fn ch(x: u8, y: u8, pct: u32) -> u8 {
        ((x as u32 * (100 - pct) + y as u32 * pct) / 100) as u8
    }
    rgb(ch(a.r(), b.r(), pct), ch(a.g(), b.g(), pct), ch(a.b(), b.b(), pct))
}

/// A scheme with a dark background and a light colour drawn on it, like the tree pictures.
const fn dark(base: Color32, cream: Color32) -> Palette {
    let black = rgb(0, 0, 0);
    Palette {
        bg: base,
        faint: mix(base, cream, 6),
        text_bg: mix(base, black, 28),
        ink: cream,
        field: cream,
        field_ink: mix(base, black, 35),
        select: mix(base, cream, 30),
        select_ink: rgb(255, 255, 255),
        shadow: mix(base, cream, 25),
        light: mix(base, cream, 40),
        button: mix(base, cream, 12),
        hover: mix(base, cream, 20),
        pressed: mix(base, black, 15),
        tip: cream,
        warn: rgb(245, 175, 110),
        dark: true,
    }
}

/// A scheme with a light background and dark text.
const fn light(bg: Color32, ink: Color32, accent: Color32) -> Palette {
    let white = rgb(255, 255, 255);
    Palette {
        bg,
        faint: mix(bg, white, 40),
        text_bg: mix(bg, white, 85),
        ink,
        field: mix(bg, white, 85),
        field_ink: ink,
        select: accent,
        select_ink: white,
        shadow: mix(bg, ink, 25),
        light: mix(bg, white, 60),
        button: mix(bg, white, 45),
        hover: mix(bg, white, 65),
        pressed: mix(bg, ink, 10),
        tip: mix(bg, white, 90),
        warn: rgb(160, 50, 0),
        dark: false,
    }
}

/// Name, the group it is listed under, and its colours.
pub const SCHEMES: &[(&str, &str, Palette)] = &[
    (
        "Classic",
        "Original",
        Palette {
            bg: rgb(236, 233, 216),
            faint: rgb(246, 244, 234),
            text_bg: rgb(255, 255, 255),
            ink: rgb(0, 0, 0),
            field: rgb(255, 255, 255),
            field_ink: rgb(0, 0, 0),
            select: rgb(81, 85, 120),
            select_ink: rgb(255, 255, 255),
            shadow: rgb(172, 168, 153),
            light: rgb(255, 255, 255),
            button: rgb(244, 242, 232),
            hover: rgb(252, 250, 240),
            pressed: rgb(220, 216, 198),
            tip: rgb(255, 255, 225),
            warn: rgb(170, 60, 0),
            dark: false,
        },
    ),
    ("Slate Blue", "Tree pictures", dark(rgb(67, 82, 106), rgb(246, 238, 226))),
    ("Navy", "Tree pictures", dark(rgb(30, 52, 99), rgb(255, 255, 255))),
    (
        "Parchment",
        "Tree pictures",
        Palette {
            bg: rgb(226, 201, 162),
            faint: rgb(233, 213, 180),
            text_bg: rgb(250, 242, 226),
            ink: rgb(66, 40, 18),
            field: rgb(250, 242, 226),
            field_ink: rgb(66, 40, 18),
            select: rgb(140, 88, 42),
            select_ink: rgb(253, 247, 235),
            shadow: rgb(176, 138, 92),
            light: rgb(245, 232, 208),
            button: rgb(238, 220, 190),
            hover: rgb(244, 230, 206),
            pressed: rgb(212, 184, 142),
            tip: rgb(252, 244, 224),
            warn: rgb(150, 50, 0),
            dark: false,
        },
    ),
    ("Blue Linen", "Tree pictures", dark(rgb(88, 110, 142), rgb(236, 231, 220))),
    ("Leather", "Tree pictures", dark(rgb(104, 52, 24), rgb(242, 224, 188))),
    ("Leaf Green", "Genealogy sites", light(rgb(243, 240, 232), rgb(52, 48, 44), rgb(104, 134, 28))),
    ("Fresh Green", "Genealogy sites", light(rgb(242, 243, 240), rgb(51, 51, 51), rgb(80, 132, 34))),
    ("Deep Green", "Genealogy sites", dark(rgb(48, 88, 38), rgb(250, 252, 246))),
    ("Sage", "Earth tones", light(rgb(199, 208, 184), rgb(40, 52, 36), rgb(88, 110, 74))),
    ("Olive", "Earth tones", dark(rgb(85, 90, 52), rgb(240, 236, 214))),
    ("Moss", "Earth tones", dark(rgb(66, 82, 50), rgb(238, 240, 220))),
    ("Forest Green", "Earth tones", dark(rgb(36, 66, 48), rgb(234, 240, 226))),
    ("Terracotta", "Earth tones", dark(rgb(150, 78, 52), rgb(250, 236, 222))),
    ("Clay", "Earth tones", light(rgb(214, 180, 156), rgb(70, 40, 28), rgb(150, 80, 50))),
    ("Burgundy", "Popular colors", dark(rgb(110, 28, 40), rgb(250, 236, 236))),
    ("Teal", "Popular colors", dark(rgb(22, 92, 100), rgb(232, 246, 244))),
    ("Plum", "Popular colors", dark(rgb(82, 46, 88), rgb(244, 234, 246))),
    ("Charcoal", "Popular colors", dark(rgb(52, 54, 58), rgb(236, 236, 232))),
    ("Midnight", "Popular colors", dark(rgb(20, 24, 40), rgb(226, 230, 240))),
    ("Sky", "Popular colors", light(rgb(200, 222, 240), rgb(20, 44, 70), rgb(40, 100, 170))),
    ("Rose", "Popular colors", light(rgb(236, 210, 212), rgb(80, 30, 40), rgb(160, 60, 80))),
    ("Ivory", "Neutrals", light(rgb(246, 242, 228), rgb(40, 38, 30), rgb(110, 104, 80))),
    ("Linen", "Neutrals", light(rgb(236, 230, 218), rgb(50, 44, 36), rgb(120, 100, 70))),
    ("Stone", "Neutrals", light(rgb(214, 212, 206), rgb(40, 40, 38), rgb(90, 94, 100))),
    ("Taupe", "Neutrals", light(rgb(200, 190, 178), rgb(50, 42, 36), rgb(110, 92, 76))),
];

static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// The colours in use now.
pub fn pal() -> &'static Palette {
    &SCHEMES[current()].2
}

pub fn current() -> usize {
    CURRENT.load(Ordering::Relaxed).min(SCHEMES.len() - 1)
}

pub fn set(index: usize) {
    CURRENT.store(index.min(SCHEMES.len() - 1), Ordering::Relaxed);
}

/// The scheme's name as saved in the settings file, so a renamed or reordered list still finds it.
pub fn name() -> &'static str {
    SCHEMES[current()].0
}

pub fn set_name(name: &str) {
    if let Some(i) = SCHEMES.iter().position(|(n, _, _)| n.eq_ignore_ascii_case(name.trim())) {
        set(i);
    }
}

//! Fonts, colours applied to the window, the program icon, and the language and colour choices.

use super::*;
use egui::{FontData, FontDefinitions, FontFamily, Stroke, vec2};
use std::sync::Arc;

// System fonts to try on each platform, in order of preference.
#[cfg(target_os = "macos")]
const PLAIN_FONTS: &[&str] = &["/System/Library/Fonts/Supplemental/Arial.ttf", "/Library/Fonts/Arial.ttf"];

#[cfg(target_os = "macos")]
const BOLD_FONTS: &[&str] = &["/System/Library/Fonts/Supplemental/Arial Bold.ttf", "/Library/Fonts/Arial Bold.ttf"];

#[cfg(target_os = "macos")]
const WIDE_FONTS: &[&str] = &["/System/Library/Fonts/Supplemental/Arial Unicode.ttf", "/Library/Fonts/Arial Unicode.ttf"];

#[cfg(target_os = "windows")]
const PLAIN_FONTS: &[&str] = &["C:\\Windows\\Fonts\\arial.ttf", "C:\\Windows\\Fonts\\segoeui.ttf"];

#[cfg(target_os = "windows")]
const BOLD_FONTS: &[&str] = &["C:\\Windows\\Fonts\\arialbd.ttf", "C:\\Windows\\Fonts\\segoeuib.ttf"];

#[cfg(target_os = "windows")]
const WIDE_FONTS: &[&str] = &[
    "C:\\Windows\\Fonts\\ARIALUNI.TTF",
    "C:\\Windows\\Fonts\\msjh.ttc",
    "C:\\Windows\\Fonts\\YuGothM.ttc",
    "C:\\Windows\\Fonts\\msgothic.ttc",
    "C:\\Windows\\Fonts\\malgun.ttf",
    "C:\\Windows\\Fonts\\msyh.ttc",
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const PLAIN_FONTS: &[&str] = &[
    "/usr/share/fonts/truetype/msttcorefonts/Arial.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf",
    "/usr/share/fonts/TTF/LiberationSans-Regular.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const BOLD_FONTS: &[&str] = &[
    "/usr/share/fonts/truetype/msttcorefonts/Arial_Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    "/usr/share/fonts/liberation-sans/LiberationSans-Bold.ttf",
    "/usr/share/fonts/TTF/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const WIDE_FONTS: &[&str] = &[
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];

/// Adds the first of these fonts that exists on this computer.
fn add_first_font(fonts: &mut FontDefinitions, name: &str, paths: &[&str], first: bool) -> bool {
    paths.iter().any(|path| add_font(fonts, name, path, first))
}

fn add_font(fonts: &mut FontDefinitions, name: &str, path: &str, first: bool) -> bool {
    let Ok(bytes) = std::fs::read(path) else { return false };
    fonts.font_data.insert(name.into(), Arc::new(FontData::from_owned(bytes)));
    let list = fonts.families.entry(FontFamily::Proportional).or_default();
    if first {
        list.insert(0, name.into());
    } else {
        list.push(name.into());
    }
    true
}

pub fn setup_look(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    // A plain sans font like the original's, plus one that covers Cyrillic and Asian names.
    add_first_font(&mut fonts, "plain", PLAIN_FONTS, true);
    // Every wide font found is added, so names in more scripts have a font behind them.
    let mut wide = Vec::new();
    for (i, path) in WIDE_FONTS.iter().enumerate() {
        let name = format!("wide{i}");
        if add_font(&mut fonts, &name, path, false) {
            wide.push(name);
        }
    }
    if add_first_font(&mut fonts, "bold", BOLD_FONTS, false) {
        // Bold names need the same wide font behind them, or Chinese, Japanese, Korean and Cyrillic
        // names would show as empty squares in the boxes. It does the job of the original's add-on
        // font package.
        let bold_list = fonts.families.entry(FontFamily::Name("bold".into())).or_default();
        bold_list.push("bold".into());
        for name in &wide {
            bold_list.push(name.clone());
        }
    } else {
        let list = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
        fonts.families.insert(FontFamily::Name("bold".into()), list);
    }
    ctx.set_fonts(fonts);
    apply_colors(ctx);
}

/// Sets the window colours from the chosen scheme. Called at start and when the scheme changes.
pub fn apply_colors(ctx: &egui::Context) {
    let p = pal();
    let mut v = if p.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = p.bg;
    v.window_fill = p.bg;
    v.extreme_bg_color = p.text_bg;
    v.faint_bg_color = p.faint;
    v.window_corner_radius = 2.into();
    v.menu_corner_radius = 1.into();
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = 2.into();
    }
    v.selection.bg_fill = p.select;
    v.selection.stroke = Stroke::new(1.0, p.select_ink);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.ink);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.ink);
    v.widgets.inactive.bg_fill = p.button;
    v.widgets.inactive.weak_bg_fill = p.button;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.shadow);
    v.widgets.hovered.weak_bg_fill = p.hover;
    v.widgets.active.weak_bg_fill = p.pressed;
    // Classic keeps the original's settings exactly; the other schemes set every state's colours.
    if crate::app::colors::current() != 0 {
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.shadow);
        v.widgets.noninteractive.bg_fill = p.bg;
        v.widgets.hovered.bg_fill = p.hover;
        v.widgets.active.bg_fill = p.pressed;
        v.widgets.open.bg_fill = p.hover;
        v.widgets.open.weak_bg_fill = p.hover;
        for w in [&mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
            w.fg_stroke = Stroke::new(1.0, p.ink);
            w.bg_stroke = Stroke::new(1.0, p.light);
        }
        v.window_stroke = Stroke::new(1.0, p.shadow);
        v.hyperlink_color = p.ink;
        v.warn_fg_color = p.warn;
    }
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.set_visuals_of(egui::Theme::Light, v.clone());
    ctx.set_visuals_of(egui::Theme::Dark, v);
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = vec2(6.0, 4.0);
        s.spacing.button_padding = vec2(7.0, 2.0);
    });
}

/// The app's icon: the white tree on blue, from assets/icon.png.
pub fn app_icon() -> egui::IconData {
    const PNG: &[u8] = include_bytes!("../../assets/icon.png");
    match image::load_from_memory(PNG) {
        Ok(img) => {
            let img = img.to_rgba8();
            let (width, height) = img.dimensions();
            egui::IconData { rgba: img.into_raw(), width, height }
        }
        Err(_) => egui::IconData::default(),
    }
}

/// Switches language straight away and remembers it for next time. It belongs to the program,
/// not the family file.
pub fn choose_language(index: usize) {
    crate::i18n::set(index);
    // A picture run is only a test, so it leaves the person's setting alone.
    if std::env::args().any(|a| a == "--screenshot") {
        return;
    }
    let _ = std::fs::create_dir_all(config_dir());
    let _ = std::fs::write(config_dir().join("language.txt"), crate::i18n::code());
}

/// Switches to a colour scheme and remembers it for next time.
pub fn choose_colors(ctx: &egui::Context, index: usize) {
    crate::app::colors::set(index);
    apply_colors(ctx);
    // A picture run is only a test, so it leaves the person's setting alone.
    if std::env::args().any(|a| a == "--screenshot") {
        return;
    }
    let _ = std::fs::create_dir_all(config_dir());
    let _ = std::fs::write(config_dir().join("colors.txt"), crate::app::colors::name());
}

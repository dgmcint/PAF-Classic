//! PAF Classic. Where things are:
//!
//! - `app`: the main window, its menus, toolbar, commands and file handling
//! - `screens`: the Family, Pedigree and Individual screens
//! - `dialogs`: the windows that open over the main screen
//! - `data`: the family file and the records in it
//! - `rules`: dates, Soundex, relationships and record checks, as the original worked them out
//! - `formats`: GEDCOM and the original's .paf files
//! - `reports`: printed reports, PDF and the family web page
//! - `i18n`: the menus and windows in other languages

// Release builds on Windows open without a console window behind the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod data;
mod dialogs;
mod formats;
mod i18n;
mod reports;
mod rules;
mod screens;

use app::colors;
use std::path::PathBuf;

fn main() -> eframe::Result {
    let mut screenshot = None;
    let mut sample = false;
    let mut file = None;
    let mut view = None;
    let mut open = None;
    let mut steps = None;
    let mut language = None;
    let mut colors = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--screenshot" => screenshot = args.next().map(PathBuf::from),
            "--sample" => sample = true,
            "--view" => view = args.next(),
            "--open" => open = args.next(),
            "--do" => steps = args.next(),
            "--lang" => language = args.next(),
            "--colors" => colors = args.next(),
            other => file = Some(PathBuf::from(other)),
        }
    }
    let taking_picture = screenshot.is_some();
    // A picture run uses Classic colours unless asked, like the language.
    match colors {
        Some(name) => colors::set_name(&name),
        None if !taking_picture => {
            if let Ok(name) = std::fs::read_to_string(app::config_dir().join("colors.txt")) {
                colors::set_name(&name);
            }
        }
        None => {}
    }
    // A picture run stays in English unless asked, so it stays comparable.
    match language {
        Some(code) => i18n::set_code(&code),
        None if !taking_picture => {
            if let Ok(code) = std::fs::read_to_string(app::config_dir().join("language.txt")) {
                i18n::set_code(&code);
            }
        }
        None => {}
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 716.0])
            .with_min_inner_size([760.0, 560.0])
            .with_title(app::APP_NAME)
            .with_icon(std::sync::Arc::new(app::app_icon())),
        ..Default::default()
    };
    eframe::run_native(
        app::APP_NAME,
        options,
        Box::new(move |cc| {
            app::setup_look(&cc.egui_ctx);
            if taking_picture {
                // A window behind others, or on a sleeping screen, may stop being drawn. This keeps
                // it drawing so the picture still arrives, and gives up rather than hang.
                let ctx = cc.egui_ctx.clone();
                std::thread::spawn(move || {
                    let began = std::time::Instant::now();
                    while began.elapsed().as_secs() < 80 {
                        ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
                        ctx.request_repaint();
                        std::thread::sleep(std::time::Duration::from_millis(16));
                    }
                    eprintln!("The window picture never arrived.");
                    std::process::exit(1);
                });
            }
            Ok(Box::new(app::App::new(screenshot, sample, file, view, open, steps)))
        }),
    )
}

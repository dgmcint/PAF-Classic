//! The `--screenshot` option: wait for the window to settle, save a picture and exit.

use super::*;
use std::sync::Arc;

impl App {
    pub(crate) fn screenshot_step(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot.clone() else { return };
        self.frames += 1;
        ctx.request_repaint();
        if self.shot_done.load(std::sync::atomic::Ordering::SeqCst) {
            std::process::exit(0);
        }
        let last_step = self.steps.iter().map(|s| s.0).max().unwrap_or(0);
        if self.frames >= 120.max(last_step + 30) && !self.screenshot_requested {
            self.screenshot_requested = true;
            let done = self.shot_done.clone();
            ctx.request_screenshot(move |shot: Arc<egui::ColorImage>| {
                let [w, h] = shot.size;
                let mut rgba = Vec::with_capacity(w * h * 4);
                for px in &shot.pixels {
                    rgba.extend_from_slice(&px.to_srgba_unmultiplied());
                }
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = image::save_buffer(&path, &rgba, w as u32, h as u32, image::ColorType::Rgba8) {
                    eprintln!("Could not save the picture: {e}");
                }
                done.store(true, std::sync::atomic::Ordering::SeqCst);
            });
        }
        // A window that never finishes must not hang whoever asked for the picture.
        if self.frames > 6000 {
            eprintln!("The window picture never arrived.");
            std::process::exit(1);
        }
    }
}

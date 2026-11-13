//! The toolbar's small drawn icons.

use crate::i18n::tr;
use egui::{Color32, FontFamily, Rect, Stroke, StrokeKind, pos2};

#[derive(Clone, Copy)]
pub enum Icon {
    Folder,
    Disk,
    Pencil,
    Page,
    Printer,
    #[allow(dead_code)]
    Couple,
    Camera,
    PersonAdd,
    CoupleAdd,
    Find,
    FindCouple,
    Tree,
    Home,
    Merge,
    Calc,
    Help,
}

/// Small drawn icons, made for this program rather than taken from the original.
pub fn draw_icon(p: &egui::Painter, r: Rect, icon: Icon, on: bool) {
    let ink = if on { Color32::from_rgb(40, 40, 40) } else { Color32::from_rgb(150, 148, 135) };
    let fill = |c: Color32| if on { c } else { Color32::from_rgb(200, 197, 182) };
    let s = Stroke::new(1.0, ink);
    let at = |x: f32, y: f32| pos2(r.left() + x * r.width(), r.top() + y * r.height());
    let box_ = |x1: f32, y1: f32, x2: f32, y2: f32| Rect::from_two_pos(at(x1, y1), at(x2, y2));
    let person = |x: f32, c: Color32| {
        p.circle(at(x, 0.28), r.width() * 0.17, fill(c), s);
        p.rect(box_(x - 0.2, 0.5, x + 0.2, 0.95), 2.0, fill(c), s, StrokeKind::Inside);
    };
    let plus = |x: f32, y: f32| {
        let green = Stroke::new(2.0, if on { Color32::from_rgb(30, 130, 50) } else { ink });
        p.line_segment([at(x - 0.14, y), at(x + 0.14, y)], green);
        p.line_segment([at(x, y - 0.14), at(x, y + 0.14)], green);
    };
    let lens = |x: f32, y: f32| {
        p.circle(at(x, y), r.width() * 0.2, fill(Color32::from_rgb(190, 220, 240)), Stroke::new(1.5, ink));
        p.line_segment([at(x + 0.14, y + 0.14), at(x + 0.34, y + 0.34)], Stroke::new(2.0, ink));
    };
    match icon {
        Icon::Folder => {
            p.rect(box_(0.0, 0.28, 0.4, 0.45), 1.0, fill(Color32::from_rgb(240, 200, 90)), s, StrokeKind::Inside);
            p.rect(box_(0.0, 0.4, 1.0, 0.92), 1.0, fill(Color32::from_rgb(245, 210, 110)), s, StrokeKind::Inside);
        }
        Icon::Disk => {
            p.rect(box_(0.05, 0.05, 0.95, 0.95), 1.0, fill(Color32::from_rgb(70, 90, 160)), s, StrokeKind::Inside);
            p.rect(box_(0.25, 0.05, 0.75, 0.4), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            p.rect(box_(0.25, 0.58, 0.75, 0.95), 0.0, fill(Color32::from_rgb(210, 210, 220)), s, StrokeKind::Inside);
        }
        Icon::Pencil => {
            p.rect(box_(0.1, 0.05, 0.8, 0.95), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            p.line_segment([at(0.35, 0.8), at(0.95, 0.2)], Stroke::new(2.5, if on { Color32::from_rgb(200, 120, 30) } else { ink }));
        }
        Icon::Page => {
            p.rect(box_(0.15, 0.03, 0.85, 0.97), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            for y in [0.25, 0.45, 0.65, 0.82] {
                p.line_segment([at(0.27, y), at(0.73, y)], s);
            }
        }
        Icon::Printer => {
            p.rect(box_(0.25, 0.05, 0.75, 0.4), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            p.rect(box_(0.03, 0.38, 0.97, 0.75), 2.0, fill(Color32::from_rgb(190, 190, 195)), s, StrokeKind::Inside);
            p.rect(box_(0.25, 0.62, 0.75, 0.97), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
        }
        Icon::Couple => {
            person(0.3, Color32::from_rgb(120, 150, 210));
            person(0.72, Color32::from_rgb(220, 130, 150));
        }
        Icon::Camera => {
            p.rect(box_(0.03, 0.3, 0.97, 0.9), 2.0, fill(Color32::from_rgb(120, 120, 130)), s, StrokeKind::Inside);
            p.rect(box_(0.33, 0.15, 0.67, 0.32), 0.0, fill(Color32::from_rgb(120, 120, 130)), s, StrokeKind::Inside);
            p.circle(at(0.5, 0.6), r.width() * 0.2, fill(Color32::from_rgb(190, 220, 240)), s);
        }
        Icon::PersonAdd => {
            person(0.36, Color32::from_rgb(120, 150, 210));
            plus(0.82, 0.25);
        }
        Icon::CoupleAdd => {
            person(0.24, Color32::from_rgb(120, 150, 210));
            person(0.62, Color32::from_rgb(220, 130, 150));
            plus(0.86, 0.2);
        }
        Icon::Find => lens(0.42, 0.42),
        Icon::FindCouple => {
            person(0.25, Color32::from_rgb(120, 150, 210));
            lens(0.6, 0.55);
        }
        Icon::Tree => {
            p.line_segment([at(0.15, 0.1), at(0.15, 0.85)], s);
            for y in [0.3, 0.6, 0.85] {
                p.line_segment([at(0.15, y), at(0.45, y)], s);
                p.rect(box_(0.45, y - 0.09, 0.95, y + 0.09), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            }
        }
        Icon::Home => {
            let roof = vec![at(0.5, 0.05), at(1.0, 0.5), at(0.0, 0.5)];
            p.add(egui::Shape::convex_polygon(roof, fill(Color32::from_rgb(180, 70, 60)), s));
            p.rect(box_(0.15, 0.5, 0.85, 0.97), 0.0, fill(Color32::from_rgb(235, 225, 190)), s, StrokeKind::Inside);
        }
        Icon::Merge => {
            p.line_segment([at(0.05, 0.15), at(0.5, 0.5)], Stroke::new(2.0, ink));
            p.line_segment([at(0.05, 0.85), at(0.5, 0.5)], Stroke::new(2.0, ink));
            p.line_segment([at(0.5, 0.5), at(0.97, 0.5)], Stroke::new(2.0, ink));
        }
        Icon::Calc => {
            p.rect(box_(0.15, 0.03, 0.85, 0.97), 1.0, fill(Color32::from_rgb(210, 210, 215)), s, StrokeKind::Inside);
            p.rect(box_(0.25, 0.12, 0.75, 0.33), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            for y in [0.48, 0.65, 0.82] {
                for x in [0.33, 0.5, 0.67] {
                    p.circle_filled(at(x, y), 1.3, ink);
                }
            }
        }
        Icon::Help => {
            p.text(r.center(), egui::Align2::CENTER_CENTER, tr("?"), egui::FontId::new(r.height() * 1.1, FontFamily::Name("bold".into())), if on { Color32::from_rgb(40, 80, 170) } else { ink });
        }
    }
}

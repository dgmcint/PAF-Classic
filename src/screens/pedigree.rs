//! The Pedigree screen.

use super::*;
use crate::i18n::{tr, trf};
use crate::app::*;
use crate::data::*;
use egui::{Align2, Key, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

/// The people in pedigree order: index 1 is the primary person, then each person's father is
/// index times two and their mother the next one.
fn tree_people(app: &App) -> [Option<u32>; 32] {
    let mut t = [None; 32];
    t[1] = app.primary;
    if t[1].is_some() {
        if let Some(m) = app.primary.and_then(|r| app.current_parents(r)).and_then(|m| app.db.marriages.get(&m)) {
            t[2] = m.husband;
            t[3] = m.wife;
        }
    }
    for i in 2..16 {
        if let Some(r) = t[i] {
            t[i * 2] = app.db.father(r);
            t[i * 2 + 1] = app.db.mother(r);
        }
    }
    t
}

const PED_W: f32 = 297.0;

/// Box tops measured from the picture, by generation.
fn pedigree_box(i: usize) -> Rect {
    const G5: [f32; 16] = [145.0, 180.0, 215.0, 250.0, 285.0, 320.0, 355.0, 390.0, 425.0, 460.0, 495.0, 531.0, 566.0, 601.0, 636.0, 671.0];
    const G4: [f32; 8] = [162.0, 232.0, 302.0, 372.0, 442.0, 512.0, 583.0, 653.0];
    const G3: [f32; 4] = [197.0, 337.0, 477.0, 618.0];
    const G2: [f32; 2] = [267.0, 548.0];
    let (x, y) = match i {
        1 => (27.0, 407.0),
        2..=3 => (65.0, G2[i - 2]),
        4..=7 => (185.0, G3[i - 4]),
        8..=15 => (379.0, G4[i - 8]),
        _ => (696.0, G5[i - 16]),
    };
    let x1 = x + PED_W;
    Rect::from_min_max(pos2(x, y), pos2(x1, y + 28.0))
}

pub fn pedigree(app: &mut App, ui: &mut egui::Ui) {
    let area = ui.max_rect();
    let fit = Fit::new(area);
    let tree = tree_people(app);
    let painter = ui.painter().clone();
    let stroke = Stroke::new(1.0, pal().ink);
    let font = plain((16.0 * fit.sy.min(fit.sx)).clamp(12.0, 22.0));
    for (x, label) in [(28.0, "1st"), (95.0, "2nd"), (208.0, "3rd"), (395.0, "4th"), (704.0, "5th")] {
        painter.text(fit.p(x, 133.0), Align2::LEFT_CENTER, tr(label), font.clone(), pal().ink);
    }
    let scaled = |b: Rect| fit.r(b.left(), b.top(), b.right(), b.bottom());
    // Connecting lines first, so the boxes sit on top.
    let (x1, x2, x3, x4) = (55.0, 174.0, 368.0, 685.0);
    let link = |parent_a: usize, parent_b: usize, x: f32, from_x: f32, from_y: f32| {
        let (a, b) = (scaled(pedigree_box(parent_a)), scaled(pedigree_box(parent_b)));
        let vx = fit.p(x, 0.0).x.round();
        painter.vline(vx, a.center().y.round()..=b.center().y.round(), stroke);
        painter.hline(vx..=a.left(), a.center().y.round(), stroke);
        painter.hline(vx..=b.left(), b.center().y.round(), stroke);
        painter.hline(from_x..=vx, from_y.round(), stroke);
    };
    let bottom_of = |i: usize| scaled(pedigree_box(i));
    link(2, 3, x1, bottom_of(1).left() + 10.0, bottom_of(1).center().y);
    for i in 2..=3 {
        let c = bottom_of(i);
        link(i * 2, i * 2 + 1, x2, fit.p(x2, 0.0).x, c.center().y);
    }
    for i in 4..=7 {
        let c = bottom_of(i);
        link(i * 2, i * 2 + 1, x3, fit.p(x3, 0.0).x, c.center().y);
    }
    for i in 8..=15 {
        let c = bottom_of(i);
        link(i * 2, i * 2 + 1, x4, c.right(), c.center().y);
    }

    let wait = app.db.prefs.info_wait_tenths as f32 / 10.0;
    let floating = app.db.prefs.floating_info;
    for i in 1..32 {
        let r = scaled(pedigree_box(i));
        let who = tree[i];
        let resp = ui.interact(r, ui.scope_id().with(("ped", i)), Sense::click());
        let sel = app.slot == Slot::Tree(i);
        let p = ui.painter();
        p.rect_filled(r, 0.0, if sel { pal().select } else { pal().field });
        p.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
        if let Some(person) = who.and_then(|w| app.db.person(w)) {
            let ink = if sel { pal().select_ink } else { pal().field_ink };
            draw_marks(p, r, person, ink, if sel { pal().select } else { pal().field });
            let f = plain(13.0);
            // One line, 4 pixels in from the left.
            p.text(pos2(r.left() + 4.0, r.center().y), Align2::LEFT_CENTER, clip(ui, &shown_name(app, person), &f, r.width() - 18.0), f.clone(), ink);
        }
        // An empty box adds a parent for the person it leads back from.
        let add = if i == 1 {
            Cmd::AddIndividual
        } else if i % 2 == 0 {
            Cmd::AddFather
        } else {
            Cmd::AddMother
        };
        let child = if i > 1 { tree[i / 2] } else { None };
        if resp.clicked() || resp.secondary_clicked() {
            app.pending.push(Cmd::SelectSlot(Slot::Tree(i), who));
        }
        if resp.double_clicked() {
            match who {
                Some(w) => {
                    let mods = ui.input(|m| m.modifiers);
                    if mods.ctrl || mods.command {
                        app.pending.push(Cmd::ToPrimary(w));
                    } else {
                        app.pending.push(Cmd::SelectSlot(Slot::Tree(i), Some(w)));
                        app.pending.push(Cmd::Edit);
                    }
                }
                None if i == 1 || child.is_some() => {
                    app.pending.push(Cmd::SelectSlot(Slot::Tree(i), child));
                    app.pending.push(add.clone());
                }
                None => {}
            }
        }
        match who {
            Some(w) => {
                resp.context_menu(|ui| other_menu(app, ui, w, i == 1));
                if floating > 0 && !sel && resp.hovered() {
                    let hovered_for = ui.input(|m| m.pointer.time_since_last_movement());
                    if hovered_for >= wait {
                        let text = info_text(app, w, floating);
                        resp.clone().on_hover_ui_at_pointer(|ui| {
                            ui.label(text);
                        });
                    } else {
                        ui.ctx().request_repaint();
                    }
                }
            }
            None => {
                if i == 1 || child.is_some() {
                    let label = if i == 1 { "Add Individual..." } else if i % 2 == 0 { "Add Father..." } else { "Add Mother..." };
                    resp.context_menu(|ui| {
                        if ui.button(label).clicked() {
                            app.pending.push(Cmd::SelectSlot(Slot::Tree(i), child));
                            app.pending.push(add.clone());
                            ui.close();
                        }
                    });
                }
            }
        }
        // Moving forward from the end of the chart.
        if i >= 16 {
            if let Some(w) = who {
                let more = app.db.person(w).map(|p| !p.parents.is_empty()).unwrap_or(false);
                if more && triangle_button(ui, pos2(r.right() + 11.0, r.center().y), 8.0, 1.0, true, "Move this person to the primary position to see more ancestors", &format!("more{i}")) {
                    app.pending.push(Cmd::ToPrimary(w));
                }
            }
        }
    }
    // Back toward a child of the primary person.
    if let Some(prim) = tree[1] {
        let r = bottom_of(1);
        let kids = app.db.children_of(prim);
        if !kids.is_empty() && triangle_button(ui, pos2(r.left() - 12.0, r.center().y), 9.0, -1.0, true, "Move to a child (C)", "back") {
            app.pending.push(Cmd::NextChild);
        }
    }
    // The locked InfoBox sits below the highlighted person.
    if let Slot::Tree(i) = app.slot {
        if let Some(w) = tree[i] {
            let level = app.db.prefs.locked_info;
            if level > 0 {
                let text = info_text(app, w, level);
                let r = scaled(pedigree_box(i));
                let galley = ui.fonts_mut(|f| f.layout(text, plain(12.0), pal().field_ink, 360.0));
                let top = if r.bottom() + galley.size().y + 12.0 > area.bottom() { r.top() - galley.size().y - 10.0 } else { r.bottom() + 1.0 };
                let left = r.left().min(area.right() - galley.size().x - 14.0);
                let bg = Rect::from_min_size(pos2(left, top), galley.size() + vec2(10.0, 8.0));
                egui::Area::new(ui.scope_id().with("infobox")).order(egui::Order::Foreground).fixed_pos(bg.min).interactable(false).show(ui.ctx(), |ui| {
                    let (rect, _) = ui.allocate_exact_size(bg.size(), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, pal().tip);
                    ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, pal().ink), StrokeKind::Inside);
                    ui.painter().galley(rect.min + vec2(5.0, 4.0), galley.clone(), pal().field_ink);
                });
            }
        }
    }
}

/// The InfoBox text: dates only, or dates and family information.
fn info_text(app: &App, who: u32, level: u8) -> String {
    let Some(p) = app.db.person(who) else { return String::new() };
    let mut out = format!("{}  (RIN {})", plain_name(&p.name), p.rin);
    let (born, died) = (event_text(&p.birth, &p.chr), event_text(&p.death, &p.burial));
    if !born.is_empty() {
        out.push_str(&format!("\n{} {born}", tr("Birth:")));
    }
    if !died.is_empty() {
        out.push_str(&format!("\n{} {died}", tr("Death:")));
    }
    if level >= 2 {
        for m in &p.spouses {
            if let Some(mar) = app.db.marriages.get(m) {
                let name = app.db.spouse_in(*m, who).map(|s| app.db.name(s)).unwrap_or_default();
                out.push_str(&format!("\n{}  {}", trf("Spouse: {name}", &[("name", &name)]), mar.event.date.trim()));
            }
        }
        let kids = app.db.children_of(who).len();
        if kids > 0 {
            out.push_str(&format!("\n{} {kids}", tr("Children:")));
        }
    }
    out
}

pub fn pedigree_keys(app: &mut App, ctx: &egui::Context) {
    let tree = tree_people(app);
    let mut i = match app.slot {
        Slot::Tree(i) => i,
        _ => 1,
    };
    let before = i;
    ctx.input(|inp| {
        if inp.key_pressed(Key::ArrowRight) {
            if i < 16 && tree[i * 2].is_some() {
                i *= 2;
            } else if i < 16 && tree[i * 2 + 1].is_some() {
                i = i * 2 + 1;
            }
        }
        if inp.key_pressed(Key::ArrowLeft) && i > 1 {
            i /= 2;
        }
        if inp.key_pressed(Key::ArrowDown) && i % 2 == 0 && tree[i + 1].is_some() {
            i += 1;
        }
        if inp.key_pressed(Key::ArrowUp) && i > 1 && i % 2 == 1 && tree[i - 1].is_some() {
            i -= 1;
        }
        if inp.key_pressed(Key::Home) && !inp.modifiers.ctrl && !inp.modifiers.command {
            i = 1;
        }
    });
    if i != before || !matches!(app.slot, Slot::Tree(_)) {
        app.pending.push(Cmd::SelectSlot(Slot::Tree(i), tree[i]));
    }
}

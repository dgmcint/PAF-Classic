//! The Family screen.

use crate::i18n::{tr, trf};
use super::*;
use crate::app::*;
use crate::rules;
use crate::data::*;
use egui::{Align2, FontId, Key, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

pub fn family(app: &mut App, ui: &mut egui::Ui) {
    let area = ui.max_rect();
    let fit = Fit::new(area);
    let primary = app.primary;
    let mrin = primary.and_then(|r| app.current_marriage(r));
    let pmrin = primary.and_then(|r| app.current_parents(r));
    let spouse = match (primary, mrin) {
        (Some(r), Some(m)) => app.db.spouse_in(m, r),
        _ => None,
    };
    let father = pmrin.and_then(|m| app.db.marriages.get(&m)).and_then(|m| m.husband);
    let mother = pmrin.and_then(|m| app.db.marriages.get(&m)).and_then(|m| m.wife);
    let children: Vec<u32> = mrin.and_then(|m| app.db.marriages.get(&m)).map(|m| m.children.clone()).unwrap_or_default();

    let label_font = plain((16.0 * fit.sy.min(fit.sx)).clamp(12.0, 22.0));
    let painter = ui.painter().clone();
    let stroke = Stroke::new(1.0, pal().ink);

    // Connecting lines between the primary person and the parents.
    let (jx, top, mid, bottom) = (fit.p(497.0, 0.0).x.round(), fit.p(0.0, 199.0).y.round(), fit.p(0.0, 244.0).y.round(), fit.p(0.0, 310.0).y.round());
    painter.hline(fit.p(463.0, 0.0).x.round()..=jx, mid, stroke);
    painter.vline(jx, top..=bottom, stroke);
    painter.hline(jx..=fit.p(532.0, 0.0).x.round(), top, stroke);
    painter.hline(jx..=fit.p(532.0, 0.0).x.round(), bottom, stroke);

    // Labels, at the positions in the original's drawing code (see the mechanics sheet).
    painter.text(fit.p(539.0, 144.0), Align2::LEFT_CENTER, tr("Parents"), label_font.clone(), pal().ink);
    painter.text(fit.p(49.0, 343.0), Align2::LEFT_CENTER, tr("Spouse"), label_font.clone(), pal().ink);
    painter.text(fit.p(49.0, 458.0), Align2::LEFT_CENTER, tr("Children"), label_font.clone(), pal().ink);
    painter.text(fit.p(614.0, 458.0), Align2::LEFT_CENTER, tr("Born"), label_font.clone(), pal().ink);
    painter.text(fit.p(736.0, 458.0), Align2::LEFT_CENTER, tr("Parent Link"), label_font.clone(), pal().ink);
    // Only drawn when LDS data is in use.
    if app.db.prefs.use_lds {
        painter.text(fit.p(878.0, 458.0), Align2::LEFT_CENTER, tr("Ordinances"), label_font.clone(), pal().ink);
    }

    // Primary person.
    let r = fit.r(42.0, 203.0, 463.0, 286.0);
    let resp = person_box(app, ui, r, primary, Slot::Primary);
    box_clicks(app, ui, &resp, primary, Slot::Primary, Cmd::AddIndividual);
    match primary {
        Some(who) => {
            resp.context_menu(|ui| other_menu(app, ui, who, true));
        }
        None => {
            resp.context_menu(|ui| {
                if ui.button(tr("Add Individual...")).clicked() {
                    app.pending.push(Cmd::AddIndividual);
                    ui.close();
                }
            });
        }
    }

    // Marriage line under the primary person.
    let r = fit.r(56.0, 291.0, 416.0, 323.0);
    let m_resp = ui.interact(r, ui.scope_id().with("marriage"), Sense::click());
    let m_sel = app.slot == Slot::Marriage;
    painter.rect_filled(r, 0.0, if m_sel { pal().select } else { pal().field });
    painter.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
    let ink = if m_sel { pal().select_ink } else { pal().field_ink };
    // The marriage line: date and place, then (D) when divorced, then the number when
    // numbers are showing.
    let text = mrin.and_then(|m| app.db.marriages.get(&m)).map(|m| {
        let mut s = event_text(&m.event, &Event::default());
        if m.divorced {
            s.push_str(" (D)");
        }
        if app.db.prefs.show_rin {
            s.push_str(&format!(" --{}", m.mrin));
        }
        s
    });
    let mfont = plain((16.0 * fit.sy.min(fit.sx)).clamp(12.0, 20.0));
    painter.text(pos2(r.left() + 5.0, r.center().y), Align2::LEFT_CENTER, tr("Marriage:"), mfont.clone(), ink);
    let label_w = ui.fonts_mut(|f| f.layout_no_wrap(format!("{} ", tr("Marriage:")), mfont.clone(), pal().ink).size().x);
    if let Some(t) = &text {
        let small = plain(12.5);
        let shown = clip(ui, t, &small, r.width() - label_w - 12.0);
        painter.text(pos2(r.left() + 5.0 + label_w + 2.0, r.center().y), Align2::LEFT_CENTER, shown, small, ink);
    }
    if m_resp.clicked() {
        app.pending.push(Cmd::SelectSlot(Slot::Marriage, primary));
    }
    if m_resp.double_clicked() {
        app.pending.push(Cmd::SelectSlot(Slot::Marriage, primary));
        app.pending.push(if mrin.is_some() { Cmd::EditMarriage } else { Cmd::AddSpouse });
    }
    m_resp.context_menu(|ui| {
        if ui.button(tr("Edit Marriage")).clicked() {
            app.pending.push(Cmd::EditMarriage);
            ui.close();
        }
        if ui.button(tr("Reports...")).clicked() {
            app.pending.push(Cmd::Reports);
            ui.close();
        }
    });

    // Spouse, with a button to see other marriages when there are several.
    let r = fit.r(42.0, 356.0, 463.0, 439.0);
    let resp = person_box(app, ui, r, spouse, Slot::Spouse);
    box_clicks(app, ui, &resp, spouse, Slot::Spouse, Cmd::AddSpouse);
    add_or_other_menu(app, &resp, spouse, Slot::Spouse, "Add Spouse...", Cmd::AddSpouse);
    if let Some(s) = spouse {
        let c = pos2(r.right() + 10.0, r.center().y);
        if triangle_button(ui, c, 9.0, 1.0, true, "Move the spouse to the primary position", "spouse") {
            app.pending.push(Cmd::ToPrimary(s));
        }
    }
    let count = primary.and_then(|r| app.db.person(r)).map(|p| p.spouses.len()).unwrap_or(0);
    if count > 1 {
        let rect = Rect::from_min_size(fit.p(120.0, 334.0), vec2(120.0, 18.0));
        let at = primary.and_then(|r| app.db.person(r)).and_then(|p| p.spouses.iter().position(|m| Some(*m) == mrin)).unwrap_or(0);
        if ui.put(rect, egui::Button::new(trf("Other marriages ({at}/{count})", &[("at", &(at + 1)), ("count", &count)])).small()).clicked() {
            app.pending.push(Cmd::ShowMarriage((at + 1) % count));
        }
    }

    // Parents, with a button to see other parents when there are several.
    for (who, slot, y0, y1, add, salt, tip) in [
        (father, Slot::Father, 158.0, 241.0, Cmd::AddFather, "father", "Move the father to the primary position (F)"),
        (mother, Slot::Mother, 267.0, 350.0, Cmd::AddMother, "mother", "Move the mother to the primary position (M)"),
    ] {
        let r = fit.r(532.0, y0, 953.0, y1);
        let resp = person_box(app, ui, r, who, slot);
        box_clicks(app, ui, &resp, who, slot, add.clone());
        let word = if slot == Slot::Father { "Add Father..." } else { "Add Mother..." };
        add_or_other_menu(app, &resp, who, slot, word, add);
        if let Some(w) = who {
            let solid = app.db.person(w).map(|p| !p.parents.is_empty()).unwrap_or(false);
            if triangle_button(ui, pos2(r.right() + 11.0, r.center().y), 10.0, 1.0, solid, tip, salt) {
                app.pending.push(Cmd::ToPrimary(w));
            }
        }
    }
    let links = primary.and_then(|r| app.db.person(r)).map(|p| p.parents.clone()).unwrap_or_default();
    if links.len() > 1 {
        let rect = Rect::from_min_size(fit.p(620.0, 136.0), vec2(120.0, 18.0));
        let next = links.iter().position(|l| Some(l.mrin) == pmrin).map(|i| (i + 1) % links.len()).unwrap_or(0);
        if ui.put(rect, egui::Button::new(trf("Other parents ({count})", &[("count", &links.len())])).small()).clicked() {
            app.pending.push(Cmd::ShowParents(next));
        }
    }

    // Children.
    let r = fit.r(42.0, 472.0, 980.0, 697.0);
    painter.rect_filled(r, 0.0, pal().field);
    painter.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
    let inner = r.shrink(1.0);
    // Eight rows fill the list, and it scrolls from the ninth child on.
    let row_h = (inner.height() / 8.0).floor();
    let mut ui_list = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)));
    let bg = ui_list.interact(inner, ui_list.scope_id().with("kids-bg"), Sense::click());
    if bg.double_clicked() {
        app.pending.push(Cmd::AddChild);
    }
    bg.context_menu(|ui| {
        if ui.button(tr("Add Child...")).clicked() {
            app.pending.push(Cmd::AddChild);
            ui.close();
        }
        if ui.button(tr("Reports...")).clicked() {
            app.pending.push(Cmd::Reports);
            ui.close();
        }
    });
    if !children.is_empty() {
        let x_born = fit.p(614.0, 0.0).x;
        let x_link = fit.p(736.0, 0.0).x;
        let x_ord = fit.p(878.0, 0.0).x;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(&mut ui_list, row_h, children.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for i in range {
                let (row, resp) = ui.allocate_exact_size(vec2(inner.width() - 12.0, row_h), Sense::click());
                let c = children[i];
                let sel = app.slot == Slot::Child(i);
                let ink = if sel { pal().select_ink } else { pal().field_ink };
                if sel {
                    ui.painter().rect_filled(row, 0.0, pal().select);
                }
                let Some(p) = app.db.person(c) else { continue };
                let cy = row.center().y;
                let solid = !p.spouses.is_empty();
                let font = plain(14.0);
                ui.painter().text(pos2(row.left() + 20.0, cy), Align2::LEFT_CENTER, format!("{}", i + 1), font.clone(), ink);
                let name_area = Rect::from_min_max(pos2(row.left() + 40.0, row.top()), pos2(x_born - 8.0, row.bottom()));
                ui.painter().text(pos2(name_area.left(), cy), Align2::LEFT_CENTER, clip(ui, &shown_name(app, p), &font, name_area.width() - 12.0), font.clone(), ink);
                draw_marks(ui.painter(), name_area, p, ink, if sel { pal().select } else { pal().field });
                ui.painter().text(pos2(x_born, cy), Align2::LEFT_CENTER, date_with_star(&p.birth, &p.chr), font.clone(), ink);
                // The column stays blank for an ordinary birth link.
                let rel = mrin.and_then(|m| p.parents.iter().find(|l| l.mrin == m)).map(|l| match l.relation.as_str() {
                    "Biological" | "Unknown" => String::new(),
                    "LDS Sealing" => "Sealing".to_string(),
                    other => other.to_string(),
                }).unwrap_or_default();
                ui.painter().text(pos2(x_link, cy), Align2::LEFT_CENTER, tr(&rel), font.clone(), ink);
                if app.db.prefs.use_lds {
                    ui.painter().text(pos2(x_ord, cy), Align2::LEFT_CENTER, rules::ordinance_flags(&app.db, c), FontId::monospace(13.0), ink);
                }
                if triangle_button(ui, pos2(row.left() + 8.0, cy), 9.0, -1.0, solid, "Move this child to the primary position", &format!("child{i}")) {
                    app.pending.push(Cmd::ToPrimary(c));
                }
                box_clicks(app, ui, &resp, Some(c), Slot::Child(i), Cmd::AddChild);
                resp.context_menu(|ui| other_menu(app, ui, c, false));
            }
        });
    }
}

fn add_or_other_menu(app: &mut App, resp: &egui::Response, who: Option<u32>, slot: Slot, add_label: &str, add: Cmd) {
    match who {
        Some(w) => {
            resp.context_menu(|ui| other_menu(app, ui, w, false));
        }
        None => {
            resp.context_menu(|ui| {
                if ui.button(add_label).clicked() {
                    app.pending.push(Cmd::SelectSlot(slot, None));
                    app.pending.push(add.clone());
                    ui.close();
                }
                if ui.button(tr("Reports...")).clicked() {
                    app.pending.push(Cmd::Reports);
                    ui.close();
                }
            });
        }
    }
}

pub fn family_keys(app: &mut App, ctx: &egui::Context) {
    let mrin = app.primary.and_then(|r| app.current_marriage(r));
    let children: Vec<u32> = mrin.and_then(|m| app.db.marriages.get(&m)).map(|m| m.children.clone()).unwrap_or_default();
    let pm = app.primary.and_then(|r| app.current_parents(r));
    let father = pm.and_then(|m| app.db.marriages.get(&m)).and_then(|m| m.husband);
    let mother = pm.and_then(|m| app.db.marriages.get(&m)).and_then(|m| m.wife);
    let spouse = match (app.primary, mrin) {
        (Some(r), Some(m)) => app.db.spouse_in(m, r),
        _ => None,
    };
    let last = children.len().saturating_sub(1);
    let next = |slot: Slot, key: Key| -> Slot {
        use Slot::*;
        match (slot, key) {
            (Primary, Key::ArrowRight) => Father,
            (Primary, Key::ArrowDown) => Marriage,
            (Marriage, Key::ArrowUp) => Primary,
            (Marriage, Key::ArrowDown) => Spouse,
            (Marriage, Key::ArrowRight) => Mother,
            (Spouse, Key::ArrowUp) => Marriage,
            (Spouse, Key::ArrowRight) => Mother,
            (Spouse, Key::ArrowDown) if !children.is_empty() => Child(0),
            (Father, Key::ArrowLeft) => Primary,
            (Father, Key::ArrowDown) => Mother,
            (Mother, Key::ArrowUp) => Father,
            (Mother, Key::ArrowLeft) => Marriage,
            (Mother, Key::ArrowDown) if !children.is_empty() => Child(0),
            (Child(0), Key::ArrowUp) => Spouse,
            (Child(i), Key::ArrowUp) => Child(i - 1),
            (Child(i), Key::ArrowDown) if i < last => Child(i + 1),
            (s, _) => s,
        }
    };
    let mut slot = app.slot;
    if matches!(slot, Slot::Tree(_) | Slot::Row) {
        slot = Slot::Primary;
    }
    let before = slot;
    ctx.input(|i| {
        for k in [Key::ArrowUp, Key::ArrowDown, Key::ArrowLeft, Key::ArrowRight] {
            if i.key_pressed(k) {
                slot = next(slot, k);
            }
        }
        if i.key_pressed(Key::Home) && !i.modifiers.ctrl && !i.modifiers.command {
            slot = Slot::Primary;
        }
        if i.key_pressed(Key::End) {
            slot = if children.is_empty() { Slot::Spouse } else { Slot::Child(last) };
        }
    });
    if slot != before {
        let who = match slot {
            Slot::Primary | Slot::Marriage => app.primary,
            Slot::Spouse => spouse,
            Slot::Father => father,
            Slot::Mother => mother,
            Slot::Child(i) => children.get(i).copied(),
            _ => None,
        };
        app.pending.push(Cmd::SelectSlot(slot, who));
    }
}

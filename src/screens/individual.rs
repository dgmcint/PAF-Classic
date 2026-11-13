//! The Individual list screen.

use super::*;
use crate::i18n::tr;
use crate::app::*;
use crate::data::*;
use egui::{Align2, Key, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

/// Which columns show, in what order, and how the list is sorted.
pub struct IndividualState {
    pub columns: Vec<String>,
    pub sort_by_rin: bool,
    pub rows: Vec<u32>,
    #[allow(dead_code)]
    pub stamp: (usize, usize),
    pub scroll_to: bool,
}

pub const ALL_COLUMNS: [&str; 14] = ["RIN", "Full Name", "Sex", "Birth Date", "Birth Place", "Christening Date", "Death Date", "Death Place", "Burial Date", "Marriage Date", "Spouse", "AFN", "Surname", "Soundex"];

impl Default for IndividualState {
    fn default() -> Self {
        IndividualState {
            columns: ["RIN", "Full Name", "Sex", "Birth Date", "Birth Place", "Death Date"].iter().map(|s| s.to_string()).collect(),
            sort_by_rin: false,
            rows: vec![],
            stamp: (usize::MAX, 0),
            scroll_to: false,
        }
    }
}

fn column_width(name: &str) -> f32 {
    match name {
        "RIN" => 56.0,
        "Full Name" | "Spouse" => 250.0,
        // Wide enough for the heading in the chosen language, such as "Geschlecht".
        "Sex" => if tr("Sex").chars().count() > 4 { 86.0 } else { 38.0 },
        "Birth Place" | "Death Place" => 220.0,
        "Surname" => 130.0,
        "AFN" | "Soundex" => 80.0,
        _ => 110.0,
    }
}

fn rebuild_rows(app: &mut App) {
    let mut rows: Vec<u32> = match &app.focus {
        Some(list) => list.iter().copied().filter(|r| app.db.people.contains_key(r)).collect(),
        None => app.db.people.keys().copied().collect(),
    };
    if !app.individual.sort_by_rin {
        rows.sort_by_key(|r| (sort_key(&app.db.people[r].name), *r));
    }
    app.individual.rows = rows;
}

pub fn individual(app: &mut App, ui: &mut egui::Ui) {
    rebuild_rows(app);
    let area = ui.max_rect().shrink(4.0);
    let row_h = 18.0;
    let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Min)));
    let ui = &mut ui;
    ui.painter().rect_filled(area, 0.0, pal().field);
    ui.painter().rect_stroke(area, 0.0, Stroke::new(1.0, pal().ink), StrokeKind::Inside);
    let columns = app.individual.columns.clone();
    let total: f32 = columns.iter().map(|c| column_width(c)).sum::<f32>().max(area.width() - 2.0);
    egui::ScrollArea::horizontal().auto_shrink([false, false]).show(ui, |ui| {
        // Header row. Click sorts by RIN or by name, right-click picks the columns.
        let (head, _) = ui.allocate_exact_size(vec2(total, 22.0), Sense::hover());
        ui.painter().rect_filled(head, 0.0, pal().bg);
        ui.painter().hline(head.x_range(), head.bottom(), Stroke::new(1.0, pal().shadow));
        let mut x = head.left();
        for col in &columns {
            let w = column_width(col);
            let r = Rect::from_min_size(pos2(x, head.top()), vec2(w, head.height()));
            let resp = ui.interact(r, ui.scope_id().with(format!("col{col}")), Sense::click());
            ui.painter().vline(r.right() - 1.0, r.y_range(), Stroke::new(1.0, pal().shadow));
            let sorted = (col == "RIN" && app.individual.sort_by_rin) || (col == "Full Name" && !app.individual.sort_by_rin);
            let text = if sorted { format!("{} ▾", tr(col)) } else { tr(col).to_string() };
            let head_font = bold(12.0);
            ui.painter().text(pos2(x + 5.0, r.center().y), Align2::LEFT_CENTER, clip(ui, &text, &head_font, w - 8.0), head_font, pal().ink);
            if resp.clicked() {
                if col == "RIN" {
                    app.individual.sort_by_rin = true;
                }
                if col == "Full Name" {
                    app.individual.sort_by_rin = false;
                }
            }
            resp.context_menu(|ui| {
                for name in ALL_COLUMNS {
                    let mut on = app.individual.columns.iter().any(|c| c == name);
                    if ui.checkbox(&mut on, tr(name)).changed() {
                        if on {
                            app.individual.columns.push(name.to_string());
                        } else if app.individual.columns.len() > 1 {
                            app.individual.columns.retain(|c| c != name);
                        }
                    }
                }
                ui.separator();
                if ui.button(tr("Move this column left")).clicked() {
                    if let Some(i) = app.individual.columns.iter().position(|c| c == col) {
                        if i > 0 {
                            app.individual.columns.swap(i, i - 1);
                        }
                    }
                    ui.close();
                }
                if ui.button(tr("Move this column right")).clicked() {
                    if let Some(i) = app.individual.columns.iter().position(|c| c == col) {
                        if i + 1 < app.individual.columns.len() {
                            app.individual.columns.swap(i, i + 1);
                        }
                    }
                    ui.close();
                }
            });
            x += w;
        }
        let rows = app.individual.rows.clone();
        let want = app.individual.scroll_to;
        app.individual.scroll_to = false;
        let selected = app.selected;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, row_h, rows.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for i in range {
                let rin = rows[i];
                let (row, resp) = ui.allocate_exact_size(vec2(total, row_h), Sense::click());
                let sel = selected == Some(rin);
                if sel {
                    ui.painter().rect_filled(row, 0.0, pal().select);
                    if want {
                        resp.scroll_to_me(None);
                    }
                }
                let ink = if sel { pal().select_ink } else { pal().field_ink };
                let Some(p) = app.db.person(rin) else { continue };
                let mut x = row.left();
                let f = plain(12.5);
                for col in &columns {
                    let w = column_width(col);
                    let mut text = crate::reports::field_value(&app.db, p, col);
                    if col == "Sex" {
                        text = tr(&format!("sex-letter:{text}")).replace("sex-letter:", "");
                    }
                    ui.painter().text(pos2(x + 5.0, row.center().y), Align2::LEFT_CENTER, clip(ui, &text, &f, w - 8.0), f.clone(), ink);
                    x += w;
                }
                if resp.clicked() || resp.secondary_clicked() {
                    app.pending.push(Cmd::SelectSlot(Slot::Row, Some(rin)));
                }
                if resp.double_clicked() {
                    app.pending.push(Cmd::SelectSlot(Slot::Row, Some(rin)));
                    app.pending.push(Cmd::Edit);
                }
                // The original's menu 106.
                resp.context_menu(|ui| {
                    let mut act = |ui: &mut egui::Ui, label: &str, c: Cmd| {
                        if ui.button(tr(label)).clicked() {
                            app.pending.push(Cmd::SelectSlot(Slot::Row, Some(rin)));
                            app.pending.push(c);
                            ui.close();
                        }
                    };
                    act(ui, "Edit", Cmd::Edit);
                    act(ui, "Notes", Cmd::Notes);
                    act(ui, "Multimedia", Cmd::Media);
                    ui.separator();
                    act(ui, "Move to Primary", Cmd::ToPrimary(rin));
                    act(ui, "Descendancy", Cmd::DescendancyList);
                    ui.separator();
                    act(ui, "Edit Marriage", Cmd::EditMarriage);
                    ui.separator();
                    act(ui, "Reports...", Cmd::Reports);
                });
            }
        });
    });
}

pub fn individual_keys(app: &mut App, ctx: &egui::Context) {
    rebuild_rows(app);
    let rows = &app.individual.rows;
    if rows.is_empty() {
        return;
    }
    let cur = app.selected.and_then(|r| rows.iter().position(|x| *x == r)).unwrap_or(0);
    let mut i = cur;
    let page = 20usize;
    ctx.input(|inp| {
        if inp.key_pressed(Key::ArrowDown) {
            i = (i + 1).min(rows.len() - 1);
        }
        if inp.key_pressed(Key::ArrowUp) {
            i = i.saturating_sub(1);
        }
        if inp.key_pressed(Key::PageDown) {
            i = (i + page).min(rows.len() - 1);
        }
        if inp.key_pressed(Key::PageUp) {
            i = i.saturating_sub(page);
        }
        if inp.key_pressed(Key::Home) {
            i = 0;
        }
        if inp.key_pressed(Key::End) {
            i = rows.len() - 1;
        }
    });
    if i != cur {
        let r = rows[i];
        app.individual.scroll_to = true;
        app.pending.push(Cmd::SelectSlot(Slot::Row, Some(r)));
    }
}

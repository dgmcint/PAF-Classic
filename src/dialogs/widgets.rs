//! Small building blocks the windows share.

use crate::i18n::tr;
use crate::app::*;
use crate::rules;
use crate::data::*;
use crate::reports::{self, Op, Report};
use egui::{Align2, Color32, RichText, Stroke, TextEdit, vec2};

pub fn window(title: &str, id: u64) -> egui::Window<'static> {
    egui::Window::new(tr(title).to_string()).id(egui::Id::unique(("dialog", id))).collapsible(false).pivot(Align2::CENTER_CENTER).default_pos(egui::pos2(512.0, 370.0))
}

pub fn text_row(ui: &mut egui::Ui, label: &str, value: &mut String, width: f32) {
    ui.label(tr(label));
    ui.add(TextEdit::singleline(value).desired_width(width));
    ui.end_row();
}

pub fn pick_combo(ui: &mut egui::Ui, id: impl egui::AsIdSalt, value: &mut String, options: &[&str], width: f32) {
    egui::ComboBox::from_id_salt(id).selected_text(tr(value.as_str()).to_string()).width(width).show_ui(ui, |ui| {
        for o in options {
            ui.selectable_value(value, o.to_string(), tr(o));
        }
    });
}

/// Citations list used on people, marriages and events.
pub fn citations_ui(ui: &mut egui::Ui, db: &Db, cites: &mut Vec<Citation>, salt: &str) {
    let mut remove = None;
    if db.sources.is_empty() {
        ui.label(tr("There are no sources in this file yet. Add them from Edit, Source List."));
    }
    for (i, c) in cites.iter_mut().enumerate() {
        ui.push_id((salt, i), |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let title = db.sources.get(&c.source).map(|s| s.title.clone()).unwrap_or_else(|| "(Select a source)".into());
                    egui::ComboBox::from_id_salt("src").selected_text(title).width(300.0).show_ui(ui, |ui| {
                        for s in db.sources.values() {
                            ui.selectable_value(&mut c.source, s.id, &s.title);
                        }
                    });
                    if ui.button(tr("Remove")).clicked() {
                        remove = Some(i);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Film/Volume/Page:"));
                    ui.add(TextEdit::singleline(&mut c.page).desired_width(280.0));
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Actual text:"));
                    ui.add(TextEdit::singleline(&mut c.actual_text).desired_width(320.0));
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Comments:"));
                    ui.add(TextEdit::singleline(&mut c.comments).desired_width(330.0));
                });
            });
        });
    }
    if let Some(i) = remove {
        cites.remove(i);
    }
    if !db.sources.is_empty() && ui.button(tr("Link a source")).clicked() {
        let first = db.sources.keys().next().copied().unwrap_or(0);
        cites.push(Citation { source: first, ..Default::default() });
    }
}

pub fn custom_ui(ui: &mut egui::Ui, list: &mut Vec<CustomEvent>, kinds: &[String], salt: &str) {
    let mut remove = None;
    egui::Grid::new((salt, "custom")).num_columns(5).spacing(vec2(6.0, 4.0)).show(ui, |ui| {
        for (i, c) in list.iter_mut().enumerate() {
            ui.push_id((salt, i), |ui| {
                egui::ComboBox::from_id_salt("kind").selected_text(tr(c.kind.as_str()).to_string()).width(130.0).show_ui(ui, |ui| {
                    for k in kinds {
                        ui.selectable_value(&mut c.kind, k.clone(), k);
                    }
                });
            });
            ui.add(TextEdit::singleline(&mut c.description).desired_width(120.0).hint_text(tr("Description")));
            ui.add(TextEdit::singleline(&mut c.date).desired_width(90.0).hint_text(tr("Date")));
            ui.add(TextEdit::singleline(&mut c.place).desired_width(150.0).hint_text(tr("Place")));
            if ui.small_button(tr("Remove")).clicked() {
                remove = Some(i);
            }
            ui.end_row();
        }
    });
    if let Some(i) = remove {
        list.remove(i);
    }
    if ui.button(tr("New Event/Attribute")).clicked() {
        list.push(CustomEvent { kind: kinds.first().cloned().unwrap_or_else(|| "Event".into()), ..Default::default() });
    }
}

pub fn date_warning(ui: &mut egui::Ui, text: &str, day_first: bool) {
    if !rules::is_standard_date(text, day_first) {
        ui.label(RichText::new(tr("The date you typed is not standard.")).color(crate::app::colors::pal().warn).size(11.0));
    }
}

pub fn order_list<T: Copy>(ui: &mut egui::Ui, list: &mut Vec<T>, label: impl Fn(&T) -> String) -> bool {
    let mut swap = None;
    let n = list.len();
    for (i, x) in list.iter().enumerate() {
        ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
                if ui.add_enabled(i > 0, egui::Button::new(tr("Up"))).clicked() {
                    swap = Some((i, i - 1));
                }
                if ui.add_enabled(i + 1 < n, egui::Button::new(tr("Down"))).clicked() {
                    swap = Some((i, i + 1));
                }
                ui.label(format!("{}. {}", i + 1, label(x)));
            });
        });
    }
    if let Some((a, b)) = swap {
        list.swap(a, b);
        return true;
    }
    false
}

pub fn person_card(ui: &mut egui::Ui, db: &Db, rin: u32) {
    let Some(p) = db.person(rin) else { return };
    ui.group(|ui| {
        ui.label(RichText::new(plain_name(&p.name)).strong());
        ui.label(format!("RIN {}   Sex {}", p.rin, p.sex.letter()));
        for (l, e) in [("Born", &p.birth), ("Christened", &p.chr), ("Died", &p.death), ("Buried", &p.burial)] {
            ui.label(format!("{l}: {} {}", e.date, e.place));
        }
        let parents = db.primary_parents(rin).map(|m| format!("{} and {}", m.husband.map(|r| db.name(r)).unwrap_or_default(), m.wife.map(|r| db.name(r)).unwrap_or_default()));
        ui.label(format!("Parents: {}", parents.unwrap_or_default()));
        for m in &p.spouses {
            if let Some(s) = db.spouse_in(*m, rin) {
                ui.label(format!("Spouse: {}", db.name(s)));
            }
        }
        if !p.afn.is_empty() {
            ui.label(format!("AFN: {}", p.afn));
        }
    });
}

pub fn save_report(r: &Report, print: bool) -> Result<(), String> {
    let pdf = reports::to_pdf(r);
    let name: String = r.title.chars().map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' }).collect();
    let target = if print {
        Some(std::env::temp_dir().join(format!("{name}.pdf")))
    } else {
        rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name(format!("{name}.pdf")).save_file()
    };
    let Some(p) = target else { return Ok(()) };
    std::fs::write(&p, pdf).map_err(|e| e.to_string())?;
    if print {
        crate::app::open_with_system(&p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn draw_page(ui: &mut egui::Ui, page: &reports::Page, scale: f32) {
    let size = vec2(reports::PAGE_W * scale, reports::PAGE_H * scale);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let p = ui.painter();
    p.rect(rect, 0.0, Color32::WHITE, Stroke::new(1.0, pal().shadow), egui::StrokeKind::Inside);
    let at = |x: f32, y: f32| rect.min + vec2(x * scale, y * scale);
    for op in &page.ops {
        match op {
            Op::Text { x, y, size, bold, text } => {
                let font = if *bold { egui::FontId::new(*size * scale, egui::FontFamily::Name("bold".into())) } else { egui::FontId::proportional(*size * scale) };
                p.text(at(*x, *y), Align2::LEFT_TOP, text, font, Color32::BLACK);
            }
            Op::Line { x1, y1, x2, y2 } => {
                p.line_segment([at(*x1, *y1), at(*x2, *y2)], Stroke::new(0.5, Color32::BLACK));
            }
            Op::Rect { x, y, w, h, shade } => {
                let r = egui::Rect::from_min_size(at(*x, *y), vec2(*w * scale, *h * scale));
                let fill = if *shade { Color32::from_gray(230) } else { Color32::WHITE };
                p.rect(r, 0.0, fill, Stroke::new(0.5, Color32::BLACK), egui::StrokeKind::Inside);
            }
        }
    }
}

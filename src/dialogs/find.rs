//! Windows for sources, finding people, filtering and merging.

//! Windows for sources, finding people, filtering and merging.
use super::*;
use crate::i18n::tr;
use crate::app::*;
use crate::rules;
use crate::data::*;
use egui::{Key, RichText, TextEdit};

impl App {
    pub fn show_find_dialog(&mut self, ctx: &egui::Context, dlg: &mut Dialog, is_top: bool) -> bool {
        let mut open = true;
        let escape = is_top && ctx.input(|i| i.key_pressed(Key::Escape));
        match dlg {
            Dialog::SourceList { id, current } => {
                let mut changed = false;
                window("Source List", *id).default_width(640.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(220.0);
                            egui::ScrollArea::vertical().id_salt("srcs").max_height(320.0).show(ui, |ui| {
                                for s in self.db.sources.values() {
                                    let title = if s.title.is_empty() { format!("(Source {})", s.id) } else { s.title.clone() };
                                    if ui.selectable_label(*current == Some(s.id), title).clicked() {
                                        *current = Some(s.id);
                                    }
                                }
                            });
                            ui.horizontal(|ui| {
                                if ui.button(tr("New")).clicked() {
                                    let sid = self.db.next_source();
                                    self.db.sources.insert(sid, Source { id: sid, title: "New source".into(), ..Default::default() });
                                    *current = Some(sid);
                                    changed = true;
                                }
                                if let Some(c) = *current {
                                    if ui.button(tr("Delete")).on_hover_text(tr("Also removes citations of this source")).clicked() {
                                        self.db.sources.remove(&c);
                                        remove_source_citations(&mut self.db, c);
                                        *current = None;
                                        changed = true;
                                    }
                                }
                            });
                        });
                        ui.separator();
                        ui.vertical(|ui| {
                            let repos: Vec<(u32, String)> = self.db.repositories.values().map(|r| (r.id, r.name.clone())).collect();
                            if let Some(s) = current.and_then(|c| self.db.sources.get_mut(&c)) {
                                egui::Grid::new("source_fields").num_columns(2).show(ui, |ui| {
                                    for (l, v) in [("Title:", &mut s.title), ("Author:", &mut s.author), ("Publication:", &mut s.publication), ("Abbreviation:", &mut s.abbreviation)] {
                                        ui.label(tr(l));
                                        changed |= ui.add(TextEdit::singleline(v).desired_width(300.0)).changed();
                                        ui.end_row();
                                    }
                                    ui.label(tr("Repository:"));
                                    let cur = s.repository.and_then(|r| repos.iter().find(|x| x.0 == r)).map(|x| x.1.clone()).unwrap_or_else(|| "(None)".into());
                                    egui::ComboBox::from_id_salt("repo").selected_text(cur).width(290.0).show_ui(ui, |ui| {
                                        changed |= ui.selectable_value(&mut s.repository, None, tr("(None)")).changed();
                                        for (r, n) in &repos {
                                            changed |= ui.selectable_value(&mut s.repository, Some(*r), n).changed();
                                        }
                                    });
                                    ui.end_row();
                                    ui.label(tr("Comments:"));
                                    changed |= ui.add(TextEdit::multiline(&mut s.comments).desired_width(300.0).desired_rows(4)).changed();
                                    ui.end_row();
                                });
                            } else {
                                ui.label(tr("Select a source, or click New."));
                            }
                        });
                    });
                    ui.separator();
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
                if changed {
                    self.touch();
                }
            }
            Dialog::RepositoryList { id, current } => {
                let mut changed = false;
                window("Repository List", *id).default_width(600.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(200.0);
                            for r in self.db.repositories.values() {
                                let title = if r.name.is_empty() { format!("(Repository {})", r.id) } else { r.name.clone() };
                                if ui.selectable_label(*current == Some(r.id), title).clicked() {
                                    *current = Some(r.id);
                                }
                            }
                            ui.horizontal(|ui| {
                                if ui.button(tr("New")).clicked() {
                                    let rid = self.db.next_repository();
                                    self.db.repositories.insert(rid, Repository { id: rid, name: "New repository".into(), ..Default::default() });
                                    *current = Some(rid);
                                    changed = true;
                                }
                                if let Some(c) = *current {
                                    if ui.button(tr("Delete")).clicked() {
                                        self.db.repositories.remove(&c);
                                        for s in self.db.sources.values_mut() {
                                            if s.repository == Some(c) {
                                                s.repository = None;
                                            }
                                        }
                                        *current = None;
                                        changed = true;
                                    }
                                }
                            });
                        });
                        ui.separator();
                        if let Some(r) = current.and_then(|c| self.db.repositories.get_mut(&c)) {
                            egui::Grid::new("repo_fields").num_columns(2).show(ui, |ui| {
                                ui.label(tr("Name:"));
                                changed |= ui.add(TextEdit::singleline(&mut r.name).desired_width(280.0)).changed();
                                ui.end_row();
                                ui.label(tr("Address:"));
                                changed |= ui.add(TextEdit::multiline(&mut r.contact.address).desired_width(280.0).desired_rows(3)).changed();
                                ui.end_row();
                                ui.label(tr("Phone:"));
                                changed |= ui.add(TextEdit::singleline(&mut r.contact.phone).desired_width(280.0)).changed();
                                ui.end_row();
                                ui.label(tr("Email:"));
                                changed |= ui.add(TextEdit::singleline(&mut r.contact.email).desired_width(280.0)).changed();
                                ui.end_row();
                            });
                        }
                    });
                    ui.separator();
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
                if changed {
                    self.touch();
                }
            }
            Dialog::EventTypes { id, marriage, new_name } => {
                let title = if *marriage { "Marriage Event List" } else { "Individual Event List" };
                let mut changed = false;
                window(title, *id).resizable(false).show(ctx, |ui| {
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        let built: &[&str] = if *marriage { &MARRIAGE_EVENTS } else { &INDIVIDUAL_EVENTS };
                        for e in built {
                            ui.label(*e);
                        }
                        let mut remove = None;
                        for (i, e) in self.db.custom_types.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(e).strong());
                                if ui.small_button(tr("Delete")).clicked() {
                                    remove = Some(i);
                                }
                            });
                        }
                        if let Some(i) = remove {
                            self.db.custom_types.remove(i);
                            changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(TextEdit::singleline(new_name).desired_width(180.0).hint_text(tr("New event name")));
                        if ui.add_enabled(!new_name.trim().is_empty(), egui::Button::new(tr("Add"))).clicked() {
                            self.db.custom_types.push(new_name.trim().to_string());
                            new_name.clear();
                            changed = true;
                        }
                    });
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
                if changed {
                    self.touch();
                }
            }
            Dialog::Picker { id, pick, filter, soundex } => {
                let pick = *pick;
                let mut chosen = None;
                window("Individual List", *id).default_width(520.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(tr("Name:"));
                        let r = ui.add(TextEdit::singleline(filter).desired_width(260.0));
                        if is_top && filter.is_empty() {
                            r.request_focus();
                        }
                        ui.checkbox(soundex, tr("Sounds like"));
                    });
                    let f = filter.to_lowercase();
                    let code = rules::soundex(filter);
                    let mut list: Vec<&Person> = self
                        .db
                        .people
                        .values()
                        .filter(|p| {
                            if f.is_empty() {
                                return true;
                            }
                            if *soundex {
                                rules::soundex(&surname_of(&p.name)) == code
                            } else {
                                plain_name(&p.name).to_lowercase().contains(&f) || p.rin.to_string() == f
                            }
                        })
                        .collect();
                    list.sort_by_key(|p| sort_key(&p.name));
                    ui.separator();
                    egui::ScrollArea::vertical().max_height(340.0).auto_shrink([false, true]).show_rows(ui, 20.0, list.len(), |ui, range| {
                        for i in range {
                            let p = list[i];
                            let text = format!("{}   {}   b. {}", plain_name(&p.name), p.rin, p.birth.date);
                            if ui.selectable_label(self.selected == Some(p.rin), text).clicked() {
                                chosen = Some(p.rin);
                            }
                        }
                    });
                    ui.separator();
                    if ui.button(tr("Cancel")).clicked() || escape {
                        open = false;
                    }
                });
                if let Some(r) = chosen {
                    open = false;
                    self.picks.push((pick, r));
                }
            }
            Dialog::MarriageList { id, filter } => {
                let mut chosen = None;
                window("Marriage List", *id).default_width(560.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(tr("Name:"));
                        ui.add(TextEdit::singleline(filter).desired_width(260.0));
                    });
                    let f = filter.to_lowercase();
                    egui::ScrollArea::vertical().max_height(340.0).auto_shrink([false, true]).show(ui, |ui| {
                        for m in self.db.marriages.values() {
                            let h = m.husband.map(|r| self.db.name(r)).unwrap_or_default();
                            let w = m.wife.map(|r| self.db.name(r)).unwrap_or_default();
                            let t = format!("{}: {h} and {w}   {}", m.mrin, m.event.date);
                            if !f.is_empty() && !t.to_lowercase().contains(&f) {
                                continue;
                            }
                            if ui.selectable_label(false, t).clicked() {
                                chosen = m.husband.or(m.wife);
                                if let Some(r) = chosen {
                                    let idx = self.db.person(r).and_then(|p| p.spouses.iter().position(|x| *x == m.mrin)).unwrap_or(0);
                                    self.shown_marriage.insert(r, idx);
                                }
                            }
                        }
                    });
                    if ui.button(tr("Cancel")).clicked() || escape {
                        open = false;
                    }
                });
                if let Some(r) = chosen {
                    self.to_primary(r);
                    open = false;
                }
            }
            Dialog::DescendancyList { id, root } => {
                let title = format!("Descendants of {}", self.db.name(*root));
                let mut chosen = None;
                window(&title, *id).default_width(460.0).show(ctx, |ui| {
                    let list = rules::descendants(&self.db, *root);
                    if list.is_empty() {
                        ui.label(tr("No descendants are recorded."));
                    }
                    egui::ScrollArea::vertical().max_height(380.0).show(ui, |ui| {
                        for (r, depth) in list {
                            ui.horizontal(|ui| {
                                ui.add_space(depth as f32 * 16.0);
                                let b = self.db.person(r).map(|p| p.birth.date.clone()).unwrap_or_default();
                                if ui.selectable_label(false, format!("{depth}. {}   {b}", self.db.name(r))).clicked() {
                                    chosen = Some(r);
                                }
                            });
                        }
                    });
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
                if let Some(r) = chosen {
                    self.to_primary(r);
                    open = false;
                }
            }
            Dialog::FindRin { id, text, marriage } => {
                let mut result = None;
                let mut err = false;
                window("Find by RIN/MRIN", *id).resizable(false).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(marriage, false, tr("RIN"));
                        ui.radio_value(marriage, true, tr("MRIN"));
                        let r = ui.add(TextEdit::singleline(text).desired_width(90.0));
                        if is_top && text.is_empty() {
                            r.request_focus();
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button(tr("OK")).clicked() || (is_top && ctx.input(|i| i.key_pressed(Key::Enter))) {
                            let n: u32 = text.trim().parse().unwrap_or(0);
                            if *marriage {
                                match self.db.marriages.get(&n) {
                                    Some(m) => {
                                        result = m.husband.or(m.wife);
                                        err = result.is_none();
                                    }
                                    None => err = true,
                                }
                            } else if self.db.people.contains_key(&n) {
                                result = Some(n);
                            } else {
                                err = true;
                            }
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if let Some(r) = result {
                    self.to_primary(r);
                    open = false;
                }
                if err {
                    self.message("Find by RIN/MRIN", "No record has that number.");
                }
            }
            Dialog::Focus { id, base, start, gens, rows, any, save_name } => {
                let dialog_id = *id;
                let mut pick_start = false;
                let mut apply = None;
                let mut save = None;
                let mut del = None;
                window("Advanced Focus/Filter", *id).default_width(600.0).show(ctx, |ui| {
                    ui.label(RichText::new(tr("Relationship filter")).strong());
                    ui.horizontal(|ui| {
                        ui.radio_value(base, 0, tr("Everyone"));
                        ui.radio_value(base, 1, tr("Ancestors of"));
                        ui.radio_value(base, 2, tr("Descendants of"));
                    });
                    if *base != 0 {
                        ui.horizontal(|ui| {
                            ui.label(self.db.name(*start));
                            if ui.button(tr("Search")).clicked() {
                                pick_start = true;
                            }
                            ui.label(tr("Generations:"));
                            ui.add(egui::DragValue::new(gens).range(1..=99));
                        });
                    }
                    ui.separator();
                    ui.label(RichText::new(tr("Field filters")).strong());
                    let mut remove = None;
                    for (i, r) in rows.iter_mut().enumerate() {
                        ui.push_id(i, |ui| {
                            ui.horizontal(|ui| {
                                pick_combo(ui, "f", &mut r.field, &FILTER_FIELDS, 130.0);
                                pick_combo(ui, "o", &mut r.op, &FILTER_OPS, 90.0);
                                if !r.op.starts_with("is empty") && !r.op.starts_with("is not") {
                                    ui.add(TextEdit::singleline(&mut r.value).desired_width(170.0));
                                }
                                if ui.small_button(tr("Remove")).clicked() {
                                    remove = Some(i);
                                }
                            });
                        });
                    }
                    if let Some(i) = remove {
                        rows.remove(i);
                    }
                    ui.horizontal(|ui| {
                        if ui.button(tr("Add Filter")).clicked() {
                            rows.push(FilterRow { field: "Full Name".into(), op: "contains".into(), value: String::new() });
                        }
                        ui.radio_value(any, false, tr("Match all (AND)"));
                        ui.radio_value(any, true, tr("Match any (OR)"));
                    });
                    ui.separator();
                    let found = filter_people(&self.db, *base, *start, *gens, rows, *any);
                    ui.label(format!("{} individuals match.", found.len()));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Show on Individual Screen")).clicked() {
                            apply = Some(found.clone());
                        }
                        ui.add(TextEdit::singleline(save_name).desired_width(140.0).hint_text(tr("List name")));
                        if ui.add_enabled(!save_name.trim().is_empty(), egui::Button::new(tr("Save List"))).clicked() {
                            save = Some((save_name.trim().to_string(), found.clone()));
                        }
                    });
                    if !self.db.focus_lists.is_empty() {
                        ui.label(RichText::new(tr("Saved lists")).strong());
                        for (n, v) in &self.db.focus_lists {
                            ui.horizontal(|ui| {
                                ui.label(format!("{n} ({})", v.len()));
                                if ui.small_button(tr("Show")).clicked() {
                                    apply = Some(v.clone());
                                }
                                if ui.small_button(tr("Delete")).clicked() {
                                    del = Some(n.clone());
                                }
                            });
                        }
                    }
                    ui.separator();
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
                if pick_start {
                    self.push(|id| Dialog::new_picker(id, PickFor::FocusStart(dialog_id)));
                }
                if let Some(n) = del {
                    self.db.focus_lists.remove(&n);
                    self.touch();
                }
                if let Some(v) = apply {
                    self.focus = Some(v);
                    self.view = View::Individual;
                    open = false;
                }
                if let Some((n, v)) = save {
                    self.db.focus_lists.insert(n, v);
                    save_name.clear();
                    self.touch();
                }
            }
            Dialog::Merge { id, pairs, at, sound_alike, by_afn, slack } => {
                let mut merged = None;
                let mut do_search = false;
                window("Match/Merge", *id).default_width(720.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(by_afn, false, tr("Match on names"));
                        ui.radio_value(by_afn, true, tr("Match on AFNs"));
                        if !*by_afn {
                            ui.checkbox(sound_alike, tr("Names that sound alike"));
                            ui.label(tr("Birth years within:"));
                            ui.add(egui::DragValue::new(slack).range(0..=50));
                        }
                        if ui.button(tr("Search")).clicked() {
                            do_search = true;
                        }
                    });
                    ui.separator();
                    if *at < pairs.len() {
                        let (a, b) = pairs[*at];
                        ui.label(format!("Possible duplicate {} of {}", *at + 1, pairs.len()));
                        ui.columns(2, |c| {
                            person_card(&mut c[0], &self.db, a);
                            person_card(&mut c[1], &self.db, b);
                        });
                        ui.horizontal(|ui| {
                            if ui.button(tr("Keep Left, Merge Right")).clicked() {
                                merged = Some((a, b));
                            }
                            if ui.button(tr("Keep Right, Merge Left")).clicked() {
                                merged = Some((b, a));
                            }
                            if ui.button(tr("Not a Match")).clicked() {
                                *at += 1;
                            }
                        });
                    } else if pairs.is_empty() {
                        ui.label(tr("Click Search to look for possible duplicates."));
                    } else {
                        ui.label(tr("That is all the possible duplicates."));
                    }
                    ui.separator();
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
                if do_search {
                    *pairs = if *by_afn {
                        let v: Vec<&Person> = self.db.people.values().filter(|p| !p.afn.trim().is_empty()).collect();
                        let mut out = vec![];
                        for (i, a) in v.iter().enumerate() {
                            for b in &v[i + 1..] {
                                if a.afn.trim().eq_ignore_ascii_case(b.afn.trim()) {
                                    out.push((a.rin, b.rin));
                                }
                            }
                        }
                        out
                    } else {
                        rules::duplicate_candidates(&self.db, *sound_alike, *slack)
                    };
                    *at = 0;
                    if pairs.is_empty() {
                        self.message("Match/Merge", "No possible duplicates were found.");
                    }
                }
                if let Some((keep, other)) = merged {
                    self.db.merge_people(keep, other);
                    self.log("Mrg Ind", keep);
                    pairs.retain(|(x, y)| *x != other && *y != other);
                    if self.primary == Some(other) {
                        self.primary = Some(keep);
                    }
                    if self.selected == Some(other) {
                        self.selected = Some(keep);
                    }
                    self.touch();
                }
            }
            Dialog::GlobalReplace { id, field, find, with, whole, case_sensitive } => {
                let mut done = None;
                window("Global Search and Replace", *id).resizable(false).show(ctx, |ui| {
                    egui::Grid::new("gr").num_columns(2).show(ui, |ui| {
                        ui.label(tr("Search in:"));
                        pick_combo(ui, "field", field, &REPLACE_FIELDS, 170.0);
                        ui.end_row();
                        text_row(ui, "Find:", find, 240.0);
                        text_row(ui, "Replace with:", with, 240.0);
                    });
                    ui.checkbox(whole, tr("Whole field only"));
                    ui.checkbox(case_sensitive, tr("Match case"));
                    ui.horizontal(|ui| {
                        if ui.add_enabled(!find.is_empty(), egui::Button::new(tr("Replace All"))).clicked() {
                            done = Some(rules::global_replace(&mut self.db, field, find, with, *whole, *case_sensitive));
                        }
                        if ui.button(tr("Close")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if let Some(n) = done {
                    self.touch();
                    self.message("Global Search and Replace", format!("Changed {n} fields."));
                }
            }
            Dialog::ConfirmMixedCase { id } => {
                window("Change Names to Mixed Case", *id).resizable(false).show(ctx, |ui| {
                    ui.label(tr("This changes names and places written in capital letters to upper and lower case, for example SMITH to Smith. Consider making a backup first. Continue?"));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Change")).clicked() {
                            // Names and places alike, as the original's pass over its text records.
                            let mut n = 0;
                            let mut fix = |text: &mut String, place: bool| {
                                let new = rules::mixed_case(text, place);
                                if new != *text {
                                    *text = new;
                                    n += 1;
                                }
                            };
                            for p in self.db.people.values_mut() {
                                fix(&mut p.name, false);
                                for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
                                    fix(&mut e.place, true);
                                }
                                for c in p.custom.iter_mut() {
                                    fix(&mut c.place, true);
                                }
                            }
                            for m in self.db.marriages.values_mut() {
                                fix(&mut m.event.place, true);
                                for c in m.custom.iter_mut() {
                                    fix(&mut c.place, true);
                                }
                            }
                            self.touch();
                            self.message("Change Names to Mixed Case", format!("Changed {n} names and places."));
                            open = false;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
            }
            _ => {}
        }
        open
    }
}

//! Windows for editing people, marriages and their links.
use crate::i18n::tr;
use crate::app::*;
use crate::calc;
use crate::dialogs::*;
use crate::model::*;
use egui::{Color32, Key, RichText, TextEdit, vec2};

impl App {
    pub fn show_people_dialog(&mut self, ctx: &egui::Context, dlg: &mut Dialog, is_top: bool) -> bool {
        let mut open = true;
        let escape = is_top && ctx.input(|i| i.key_pressed(Key::Escape));
        match dlg {
            Dialog::Message { id, title, text } => {
                window(title, *id).resizable(false).show(ctx, |ui| {
                    ui.set_max_width(440.0);
                    ui.label(text.as_str());
                    ui.add_space(8.0);
                    if ui.button(tr("OK")).clicked() || escape {
                        open = false;
                    }
                });
            }
            Dialog::EditPerson { id, draft, is_new, link, panel, cite_on } => {
                let title = if *is_new { "Add Individual" } else { "Edit Individual" };
                let df = self.db.prefs.day_first_numeric;
                let use_lds = self.db.prefs.use_lds;
                let mut custom_kinds: Vec<String> = INDIVIDUAL_EVENTS.iter().map(|s| s.to_string()).collect();
                custom_kinds.extend(self.db.custom_types.iter().cloned());
                let mut save = false;
                window(title, *id).default_width(620.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        for (n, t) in [(0u8, "Individual"), (1, "Notes"), (2, "Sources"), (3, "Address"), (4, "Events")] {
                            ui.selectable_value(panel, n, tr(t));
                        }
                    });
                    ui.separator();
                    match *panel {
                        0 => {
                            egui::Grid::new("person_fields").num_columns(2).spacing(vec2(8.0, 5.0)).show(ui, |ui| {
                                ui.label(tr("Full Name:"));
                                ui.add(TextEdit::singleline(&mut draft.name).desired_width(440.0).hint_text(tr("Given /Surname/ Suffix")));
                                ui.end_row();
                                ui.label(tr("Sex:"));
                                ui.horizontal(|ui| {
                                    ui.radio_value(&mut draft.sex, Sex::Male, tr("Male"));
                                    ui.radio_value(&mut draft.sex, Sex::Female, tr("Female"));
                                    ui.radio_value(&mut draft.sex, Sex::Unknown, tr("Unknown"));
                                });
                                ui.end_row();
                                text_row(ui, "Title (prefix):", &mut draft.title, 440.0);
                                text_row(ui, "Nickname:", &mut draft.nickname, 440.0);
                                text_row(ui, "Also Known As:", &mut draft.aka, 440.0);
                            });
                            ui.add_space(4.0);
                            egui::Grid::new("person_events").num_columns(3).spacing(vec2(8.0, 4.0)).show(ui, |ui| {
                                ui.label(tr(""));
                                ui.label(RichText::new(tr("Date")).strong());
                                ui.label(RichText::new(tr("Place")).strong());
                                ui.end_row();
                                let rows: [(&str, &mut Event); 4] = [("Birth:", &mut draft.birth), ("Christening:", &mut draft.chr), ("Death:", &mut draft.death), ("Burial:", &mut draft.burial)];
                                for (l, e) in rows {
                                    let mark = if e.citations.is_empty() { tr(l).to_string() } else { format!("{} s", tr(l)) };
                                    ui.label(mark);
                                    ui.vertical(|ui| {
                                        ui.add(TextEdit::singleline(&mut e.date).desired_width(130.0).min_size(vec2(130.0, 0.0)));
                                        date_warning(ui, &e.date, df);
                                    });
                                    ui.add(TextEdit::singleline(&mut e.place).desired_width(320.0));
                                    ui.end_row();
                                }
                                if use_lds {
                                    ui.label(tr("Baptism:"));
                                    ui.add(TextEdit::singleline(&mut draft.baptism.date).desired_width(130.0).min_size(vec2(130.0, 0.0)));
                                    ui.add(TextEdit::singleline(&mut draft.baptism.temple).desired_width(320.0).hint_text(tr("Temple")));
                                    ui.end_row();
                                    ui.label(tr("Endowment:"));
                                    ui.add(TextEdit::singleline(&mut draft.endowment.date).desired_width(130.0).min_size(vec2(130.0, 0.0)));
                                    ui.add(TextEdit::singleline(&mut draft.endowment.temple).desired_width(320.0).hint_text(tr("Temple")));
                                    ui.end_row();
                                }
                                ui.label(tr("AFN:"));
                                // Locked unless the preference allows editing it, as in the original.
                                ui.add_enabled(self.db.prefs.allow_afn_edit, TextEdit::singleline(&mut draft.afn).desired_width(130.0).min_size(vec2(130.0, 0.0)));
                                if draft.rin != 0 {
                                    ui.label(format!("RIN: {}", draft.rin));
                                }
                                ui.end_row();
                            });
                            ui.horizontal(|ui| {
                                ui.label(tr("Ditto from:"));
                                let sel = self.selected;
                                let choices = [("Selected", sel), ("Father", sel.and_then(|r| self.db.father(r))), ("Mother", sel.and_then(|r| self.db.mother(r))), ("Primary", self.primary)];
                                for (name, who) in choices {
                                    if ui.add_enabled(who.is_some(), egui::Button::new(tr(name))).on_hover_text(tr("Fill the empty surname and place fields from this person")).clicked() {
                                        if let Some(p) = who.and_then(|r| self.db.person(r)) {
                                            let fill = |a: &mut String, b: &str| {
                                                if a.trim().is_empty() {
                                                    *a = b.to_string();
                                                }
                                            };
                                            if surname_of(&draft.name).is_empty() {
                                                draft.name = format!("{} /{}/", given_of(&draft.name), surname_of(&p.name));
                                            }
                                            fill(&mut draft.birth.place, &p.birth.place);
                                            fill(&mut draft.chr.place, &p.chr.place);
                                            fill(&mut draft.death.place, &p.death.place);
                                            fill(&mut draft.burial.place, &p.burial.place);
                                        }
                                    }
                                }
                            });
                        }
                        1 => {
                            ui.add(TextEdit::multiline(&mut draft.notes).desired_width(600.0).desired_rows(16));
                        }
                        2 => {
                            ui.horizontal(|ui| {
                                for (n, t) in [(0u8, "Individual"), (1, "Birth"), (2, "Christening"), (3, "Death"), (4, "Burial")] {
                                    ui.selectable_value(cite_on, n, tr(t));
                                }
                            });
                            let list = match *cite_on {
                                1 => &mut draft.birth.citations,
                                2 => &mut draft.chr.citations,
                                3 => &mut draft.death.citations,
                                4 => &mut draft.burial.citations,
                                _ => &mut draft.citations,
                            };
                            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| citations_ui(ui, &self.db, list, "person"));
                        }
                        3 => {
                            egui::Grid::new("person_contact").num_columns(2).show(ui, |ui| {
                                ui.label(tr("Address:"));
                                ui.add(TextEdit::multiline(&mut draft.contact.address).desired_width(440.0).desired_rows(4));
                                ui.end_row();
                                text_row(ui, "Phone:", &mut draft.contact.phone, 440.0);
                                text_row(ui, "Email:", &mut draft.contact.email, 440.0);
                            });
                        }
                        _ => {
                            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| custom_ui(ui, &mut draft.custom, &custom_kinds, "person"));
                        }
                    }
                    ui.separator();
                    // Live checks run on a copy with this draft in place, so warnings match what will be saved.
                    let rin = if draft.rin == 0 { u32::MAX } else { draft.rin };
                    let warnings = {
                        // Swapped in and back out, so a large family isn't copied on every frame.
                        let mut copy = (**draft).clone();
                        copy.rin = rin;
                        let old = self.db.people.insert(rin, copy);
                        let found = calc::check_person(&self.db, rin);
                        match old {
                            Some(p) => {
                                self.db.people.insert(rin, p);
                            }
                            None => {
                                self.db.people.remove(&rin);
                            }
                        }
                        found
                    };
                    if !warnings.is_empty() {
                        egui::CollapsingHeader::new(RichText::new(format!("{} possible problems", warnings.len())).color(Color32::from_rgb(170, 60, 0))).show(ui, |ui| {
                            for w in &warnings {
                                ui.label(tr(w));
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        if ui.button(tr("Save")).clicked() {
                            save = true;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if save {
                    if self.db.prefs.auto_slash {
                        draft.name = with_surname_marks(&draft.name);
                    }
                    let t = calc::today();
                    draft.changed = calc::format_date(t.0, t.1, t.2);
                    let p = (**draft).clone();
                    let mut notices: Vec<String> = Vec::new();
                    if self.db.prefs.verify_new_places {
                        let known = self.db.all_places();
                        for e in [&p.birth, &p.chr, &p.death, &p.burial] {
                            let place = e.place.trim();
                            if !place.is_empty() && !known.iter().any(|k| k.eq_ignore_ascii_case(place)) {
                                notices.push(format!("New place: {place}. Please check its spelling."));
                            }
                        }
                    }
                    if *is_new && self.db.prefs.check_duplicates {
                        let name = plain_name(&p.name).to_lowercase();
                        for other in self.db.people.values().filter(|o| !name.is_empty() && plain_name(&o.name).to_lowercase() == name) {
                            notices.push(format!("{} (RIN {}) has the same name. Match/Merge can combine them if this is one person.", plain_name(&other.name), other.rin));
                        }
                    }
                    if !notices.is_empty() {
                        self.message("Please check", notices.join("\n"));
                    }
                    if *is_new {
                        let was_empty = self.primary.is_none();
                        let rin = self.db.add_person(p);
                        self.log("Add Ind", rin);
                        let link = *link;
                        self.apply_link(link, rin);
                        if was_empty || matches!(link, Link::None) {
                            self.to_primary(rin);
                        } else {
                            self.selected = Some(rin);
                        }
                    } else {
                        let rin = p.rin;
                        self.db.people.insert(rin, p);
                        self.log("Mod Ind", rin);
                    }
                    self.touch();
                    open = false;
                }
            }
            Dialog::EditMarriage { id, draft, notes_open, sources_open } => {
                let mut save = false;
                let mut delete = false;
                let use_lds = self.db.prefs.use_lds;
                let mut kinds: Vec<String> = MARRIAGE_EVENTS.iter().map(|s| s.to_string()).collect();
                kinds.extend(self.db.custom_types.iter().cloned());
                let df = self.db.prefs.day_first_numeric;
                window("Edit Marriage", *id).default_width(560.0).show(ctx, |ui| {
                    let name = |r: Option<u32>| r.map(|x| self.db.name(x)).unwrap_or_default();
                    ui.label(format!("Husband: {}", name(draft.husband)));
                    ui.label(format!("Wife: {}", name(draft.wife)));
                    ui.checkbox(&mut draft.divorced, tr("Divorced"));
                    ui.separator();
                    egui::Grid::new("marriage_fields").num_columns(2).spacing(vec2(8.0, 5.0)).show(ui, |ui| {
                        ui.label(tr("Marriage date:"));
                        ui.vertical(|ui| {
                            ui.add(TextEdit::singleline(&mut draft.event.date).desired_width(160.0).min_size(vec2(160.0, 0.0)));
                            date_warning(ui, &draft.event.date, df);
                        });
                        ui.end_row();
                        text_row(ui, "Marriage place:", &mut draft.event.place, 400.0);
                        if use_lds {
                            text_row(ui, "Sealed to spouse:", &mut draft.sealing.date, 160.0);
                            text_row(ui, "Temple:", &mut draft.sealing.temple, 160.0);
                        }
                    });
                    custom_ui(ui, &mut draft.custom, &kinds, "marriage");
                    ui.horizontal(|ui| {
                        ui.toggle_value(notes_open, tr("Notes"));
                        ui.toggle_value(sources_open, tr("Sources"));
                    });
                    if *notes_open {
                        ui.add(TextEdit::multiline(&mut draft.notes).desired_width(520.0).desired_rows(6));
                    }
                    if *sources_open {
                        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| citations_ui(ui, &self.db, &mut draft.citations, "marriage"));
                    }
                    ui.label(format!("MRIN: {}", draft.mrin));
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(tr("Save")).clicked() {
                            save = true;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                        if ui.button(tr("Delete")).on_hover_text(tr("This marriage will be removed and unlinked from husband, wife and children.")).clicked() {
                            delete = true;
                        }
                    });
                });
                if save {
                    let m = (**draft).clone();
                    self.log("Mod Mar", m.mrin);
                    self.db.marriages.insert(m.mrin, m);
                    self.touch();
                    open = false;
                }
                if delete {
                    self.db.delete_marriage(draft.mrin);
                    self.log("Del Mar", draft.mrin);
                    self.touch();
                    open = false;
                }
            }
            Dialog::AddOrSelect { id, link, sex, name } => {
                let (link, sex, name) = (*link, *sex, name.clone());
                window("Add or Select Individual", *id).resizable(false).show(ctx, |ui| {
                    ui.label(tr("Add a new person, or link someone already in this file?"));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button(tr("Add New")).clicked() {
                            self.open_new_person(link, sex, name.clone());
                            open = false;
                        }
                        if ui.button(tr("Select Existing")).clicked() {
                            self.push(|id| Dialog::new_picker(id, PickFor::Link(link)));
                            open = false;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
            }
            Dialog::Notes { id, rin, text, find, with } => {
                let title = format!("Edit Notes: {}", self.db.name(*rin));
                let mut save = false;
                window(&title, *id).default_width(620.0).show(ctx, |ui| {
                    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                        ui.add(TextEdit::multiline(text).desired_width(600.0).desired_rows(18));
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr("Find:"));
                        ui.add(TextEdit::singleline(find).desired_width(150.0));
                        ui.label(tr("Replace with:"));
                        ui.add(TextEdit::singleline(with).desired_width(150.0));
                        if ui.add_enabled(!find.is_empty(), egui::Button::new(tr("Replace All"))).clicked() {
                            *text = text.replace(find.as_str(), with.as_str());
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button(tr("Save")).clicked() {
                            save = true;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if save {
                    if let Some(p) = self.db.people.get_mut(rin) {
                        p.notes = text.clone();
                    }
                    let r = *rin;
                    self.log("Mod Nts", r);
                    self.touch();
                    open = false;
                }
            }
            Dialog::MediaList { id, rin, current } => {
                let rin = *rin;
                let title = format!("Multimedia: {}", self.db.name(rin));
                let mut changed = false;
                let mut add_file = None;
                window(&title, *id).default_width(560.0).show(ctx, |ui| {
                    let count = self.db.person(rin).map(|p| p.media.len()).unwrap_or(0);
                    if count == 0 {
                        ui.label(tr("No pictures or files have been added for this individual."));
                    }
                    if *current >= count {
                        *current = count.saturating_sub(1);
                    }
                    if count > 0 {
                        ui.horizontal(|ui| {
                            if ui.add_enabled(*current > 0, egui::Button::new(tr("Previous"))).clicked() {
                                *current -= 1;
                            }
                            ui.label(format!("{} of {count}", *current + 1));
                            if ui.add_enabled(*current + 1 < count, egui::Button::new(tr("Next"))).clicked() {
                                *current += 1;
                            }
                        });
                        let file = self.db.people[&rin].media[*current].file.clone();
                        match self.texture_for(ctx, &file) {
                            Some(tex) => {
                                let sz = tex.size_vec2();
                                let scale = (500.0 / sz.x).min(320.0 / sz.y).min(1.0);
                                ui.image((tex.id(), sz * scale));
                            }
                            None => {
                                ui.label(format!("This picture could not be shown:\n{file}"));
                            }
                        }
                        let m = &mut self.db.people.get_mut(&rin).unwrap().media[*current];
                        egui::Grid::new("media_fields").num_columns(2).show(ui, |ui| {
                            ui.label(tr("Caption:"));
                            changed |= ui.add(TextEdit::singleline(&mut m.caption).desired_width(420.0)).changed();
                            ui.end_row();
                            ui.label(tr("Description:"));
                            changed |= ui.add(TextEdit::singleline(&mut m.description).desired_width(420.0)).changed();
                            ui.end_row();
                        });
                        changed |= ui.checkbox(&mut m.in_scrapbook, tr("Include in the scrapbook")).changed();
                    }
                    ui.horizontal(|ui| {
                        if ui.button(tr("Add Picture or File...")).clicked() {
                            add_file = rfd::FileDialog::new().add_filter("Pictures", &["jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff"]).add_filter("All files", &["*"]).pick_file();
                        }
                        if count > 0 && ui.button(tr("Remove")).clicked() {
                            self.db.people.get_mut(&rin).unwrap().media.remove(*current);
                            changed = true;
                        }
                        if ui.button(tr("Close")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if let Some(f) = add_file {
                    if let Some(p) = self.db.people.get_mut(&rin) {
                        p.media.push(Media { file: f.display().to_string(), in_scrapbook: true, ..Default::default() });
                        *current = p.media.len() - 1;
                    }
                    changed = true;
                }
                if changed {
                    self.touch();
                }
            }
            Dialog::Unlink { id, rin } => {
                let rin = *rin;
                let title = format!("Unlink {}", self.db.name(rin));
                window(&title, *id).resizable(false).show(ctx, |ui| {
                    ui.label(tr("Choose the link to remove. The individual stays in the file."));
                    let Some(p) = self.db.person(rin).cloned() else {
                        open = false;
                        return;
                    };
                    if p.spouses.is_empty() && p.parents.is_empty() {
                        ui.label(tr("This individual is not linked to anyone."));
                    }
                    for m in &p.spouses {
                        let other = self.db.spouse_in(*m, rin).map(|r| self.db.name(r)).unwrap_or_else(|| "Unknown spouse".into());
                        if ui.button(format!("Unlink from spouse: {other}")).clicked() {
                            self.db.unlink_spouse(*m, rin);
                            self.log("Unl Sp", rin);
                            self.touch();
                            open = false;
                        }
                    }
                    for l in &p.parents {
                        let Some(m) = self.db.marriages.get(&l.mrin) else { continue };
                        let dad = m.husband.map(|r| self.db.name(r)).unwrap_or_default();
                        let mum = m.wife.map(|r| self.db.name(r)).unwrap_or_default();
                        if ui.button(format!("Unlink from parents: {dad} and {mum}")).clicked() {
                            self.db.unlink_child(l.mrin, rin);
                            self.log("Unl Ch", rin);
                            self.touch();
                            open = false;
                        }
                    }
                    ui.add_space(6.0);
                    if ui.button(tr("Cancel")).clicked() || escape {
                        open = false;
                    }
                });
            }
            Dialog::ConfirmDeletePerson { id, rin } => {
                let rin = *rin;
                let name = self.db.name(rin);
                window("Delete Individual", *id).resizable(false).show(ctx, |ui| {
                    ui.label(format!("Delete {name} (RIN {rin})? This removes the record from the file and unlinks it from the family."));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button(tr("Delete")).clicked() {
                            self.db.delete_person(rin);
                            self.log("Del Ind", rin);
                            self.touch();
                            open = false;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
            }
            Dialog::Parents { id, rin } => {
                let rin = *rin;
                let title = format!("Parents of {}", self.db.name(rin));
                let mut changed = false;
                window(&title, *id).default_width(520.0).show(ctx, |ui| {
                    let Some(mut links) = self.db.person(rin).map(|p| p.parents.clone()) else {
                        open = false;
                        return;
                    };
                    if links.is_empty() {
                        ui.label(tr("No parents are linked. Use the Add menu to add them."));
                    }
                    let mut make_first = None;
                    for (i, l) in links.iter_mut().enumerate() {
                        ui.push_id(i, |ui| {
                            ui.group(|ui| {
                                let (dad, mum) = self.db.marriages.get(&l.mrin).map(|m| (m.husband.map(|r| self.db.name(r)).unwrap_or_default(), m.wife.map(|r| self.db.name(r)).unwrap_or_default())).unwrap_or_default();
                                ui.label(format!("{dad} and {mum}  (MRIN {}){}", l.mrin, if i == 0 { "  Primary" } else { "" }));
                                ui.horizontal(|ui| {
                                    ui.label(tr("Relationship to parents:"));
                                    let before = l.relation.clone();
                                    pick_combo(ui, "rel", &mut l.relation, &PARENT_RELATIONS, 110.0);
                                    changed |= before != l.relation;
                                    if i > 0 && ui.button(tr("Make Primary")).clicked() {
                                        make_first = Some(i);
                                    }
                                });
                                if self.db.prefs.use_lds {
                                    ui.horizontal(|ui| {
                                        ui.label(tr("Sealed to parents:"));
                                        changed |= ui.add(TextEdit::singleline(&mut l.sealed.date).desired_width(110.0).min_size(vec2(110.0, 0.0))).changed();
                                        ui.label(tr("Temple:"));
                                        changed |= ui.add(TextEdit::singleline(&mut l.sealed.temple).desired_width(80.0)).changed();
                                    });
                                }
                            });
                        });
                    }
                    if let Some(i) = make_first {
                        let l = links.remove(i);
                        links.insert(0, l);
                        changed = true;
                    }
                    if changed {
                        self.db.people.get_mut(&rin).unwrap().parents = links;
                    }
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
                if changed {
                    self.touch();
                }
            }
            Dialog::OrderSpouses { id, rin } => {
                let rin = *rin;
                let mut changed = false;
                window("Order Spouses", *id).resizable(false).show(ctx, |ui| {
                    let Some(mut list) = self.db.person(rin).map(|p| p.spouses.clone()) else {
                        open = false;
                        return;
                    };
                    changed = order_list(ui, &mut list, |m| self.db.spouse_in(*m, rin).map(|r| self.db.name(r)).unwrap_or_else(|| "Unknown spouse".into()));
                    if changed {
                        self.db.people.get_mut(&rin).unwrap().spouses = list;
                    }
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
                if changed {
                    self.touch();
                }
            }
            Dialog::OrderChildren { id, mrin } => {
                let mrin = *mrin;
                let mut changed = false;
                window("Order Children", *id).resizable(false).show(ctx, |ui| {
                    let Some(mut list) = self.db.marriages.get(&mrin).map(|m| m.children.clone()) else {
                        open = false;
                        return;
                    };
                    changed = order_list(ui, &mut list, |r| {
                        let b = self.db.person(*r).map(|p| p.birth.date.clone()).unwrap_or_default();
                        format!("{}   {b}", self.db.name(*r))
                    });
                    ui.horizontal(|ui| {
                        if ui.button(tr("Sort by Birth Date")).clicked() {
                            let df = self.db.prefs.day_first_numeric;
                            list.sort_by_key(|r| self.db.person(*r).and_then(|p| calc::date_value(&p.birth.date, df)).unwrap_or(i64::MAX));
                            changed = true;
                        }
                        if ui.button(tr("Close")).clicked() || escape {
                            open = false;
                        }
                    });
                    if changed {
                        self.db.marriages.get_mut(&mrin).unwrap().children = list;
                    }
                });
                if changed {
                    self.touch();
                }
            }
            _ => {}
        }
        open
    }
}

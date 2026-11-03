//! The three main screens. Positions are the pixels measured from the person's pictures of the
//! original at 1024 by 742; they scale with the window as the original's screens do.
use crate::i18n::tr;
use crate::app::*;
use crate::calc;
use crate::model::*;
use egui::{Align2, Color32, FontFamily, FontId, Key, Pos2, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

/// Maps the pictures' coordinates onto whatever size the screen area is now.
struct Fit {
    origin: Pos2,
    sx: f32,
    sy: f32,
}

impl Fit {
    /// The pictures' view area runs from y = 100 to y = 720.
    fn new(area: Rect) -> Self {
        Fit { origin: area.min, sx: area.width() / 1024.0, sy: area.height() / 620.0 }
    }
    fn p(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.origin.x + x * self.sx, self.origin.y + (y - 100.0) * self.sy)
    }
    fn r(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
        Rect::from_min_max(self.p(x0, y0), self.p(x1, y1)).round_to_pixels(1.0)
    }
}

trait RoundPixels {
    fn round_to_pixels(self, ppp: f32) -> Rect;
}

impl RoundPixels for Rect {
    fn round_to_pixels(self, _ppp: f32) -> Rect {
        Rect::from_min_max(pos2(self.min.x.round(), self.min.y.round()), pos2(self.max.x.round(), self.max.y.round()))
    }
}

fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

fn plain(size: f32) -> FontId {
    FontId::proportional(size)
}

/// Fits text into a width, as the original clips names and places.
fn clip(ui: &egui::Ui, text: &str, font: &FontId, width: f32) -> String {
    let w = |s: &str| ui.fonts_mut(|f| f.layout_no_wrap(s.to_string(), font.clone(), INK).size().x);
    if w(text) <= width {
        return text.to_string();
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let s: String = chars.iter().collect::<String>() + "…";
        if w(&s) <= width {
            return s;
        }
    }
    String::new()
}

/// Birth or death as the original's boxes show it: "date - place". When the
/// event itself is empty, the christening or burial is used with an asterisk in front.
fn event_text(main: &Event, other: &Event) -> String {
    let join = |e: &Event| match (e.date.trim(), e.place.trim()) {
        ("", p) => p.to_string(),
        (d, "") => d.to_string(),
        (d, p) => format!("{d} - {p}"),
    };
    let first = join(main);
    if !first.is_empty() {
        first
    } else if !join(other).is_empty() {
        format!("*{}", join(other))
    } else {
        String::new()
    }
}

/// Just the date, for the children's Born column.
fn date_with_star(main: &Event, other: &Event) -> String {
    if !main.date.trim().is_empty() {
        main.date.trim().to_string()
    } else if !other.date.trim().is_empty() {
        format!("*{}", other.date.trim())
    } else {
        String::new()
    }
}

/// The name as shown, with the number after it only when that display is turned on.
fn shown_name(app: &App, p: &Person) -> String {
    if app.db.prefs.show_rin { format!("{} -{}", plain_name(&p.name), p.rin) } else { plain_name(&p.name) }
}

/// The corner marks: a filled triangle for notes or sources and a small square for
/// multimedia. With both, the square is drawn in the box's colour on top of the triangle.
fn draw_marks(painter: &egui::Painter, rect: Rect, p: &Person, color: Color32, back: Color32) {
    let has_cite = !p.citations.is_empty() || [&p.birth, &p.chr, &p.death, &p.burial].iter().any(|e| !e.citations.is_empty());
    let note = !p.notes.trim().is_empty() || has_cite;
    let (right, top) = (rect.right(), rect.top());
    if note {
        painter.add(egui::Shape::convex_polygon(vec![pos2(right - 9.0, top + 1.0), pos2(right - 1.0, top + 1.0), pos2(right - 1.0, top + 7.0)], color, Stroke::NONE));
    }
    if !p.media.is_empty() {
        let square = Rect::from_min_max(pos2(right - 5.0, top + 1.0), pos2(right - 1.0, top + 4.0));
        painter.rect_filled(square, 0.0, if note { back } else { color });
    }
}

/// One person's box on the Family screen: the space inside a 4 by 3 margin is split
/// into three equal rows, for the name, the birth and the death.
fn person_box(app: &App, ui: &mut egui::Ui, rect: Rect, who: Option<u32>, slot: Slot) -> egui::Response {
    let id = ui.scope_id().with(format!("box{slot:?}"));
    let resp = ui.interact(rect, id, Sense::click());
    let sel = app.slot == slot;
    let painter = ui.painter();
    let back = if sel { SELECT } else { Color32::WHITE };
    painter.rect_filled(rect, 0.0, back);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, INK), StrokeKind::Inside);
    let Some(p) = who.and_then(|r| app.db.person(r)) else { return resp };
    let ink = if sel { Color32::WHITE } else { INK };
    let inner = rect.shrink2(vec2(4.0, 3.0));
    let row = inner.height() / 3.0;
    let font = plain(15.0);
    let width = inner.width() - 8.0;
    let texts = [shown_name(app, p), event_text(&p.birth, &p.chr), event_text(&p.death, &p.burial)];
    for (i, text) in texts.iter().enumerate() {
        let y = inner.top() + row * (i as f32 + 0.5);
        painter.text(pos2(inner.left(), y), Align2::LEFT_CENTER, clip(ui, text, &font, width), font.clone(), ink);
    }
    draw_marks(painter, rect, p, ink, back);
    resp
}

/// A filled triangle button means there is more to see in that direction.
fn triangle_button(ui: &mut egui::Ui, center: Pos2, size: f32, point: f32, solid: bool, tip: &str, salt: &str) -> bool {
    let rect = Rect::from_center_size(center, vec2(size + 4.0, size + 4.0));
    let resp = ui.interact(rect, ui.scope_id().with(("tri", salt)), Sense::click()).on_hover_text(tr(tip));
    let (a, b, c) = if point > 0.0 {
        (center + vec2(-size / 2.0, -size / 2.0), center + vec2(-size / 2.0, size / 2.0), center + vec2(size / 2.0, 0.0))
    } else {
        (center + vec2(size / 2.0, -size / 2.0), center + vec2(size / 2.0, size / 2.0), center + vec2(-size / 2.0, 0.0))
    };
    let fill = if solid { INK } else { SHADOW };
    ui.painter().add(egui::Shape::convex_polygon(vec![a, b, c], fill, Stroke::new(1.0, INK)));
    resp.clicked()
}

/// The right-click menu for a person, taken from the original's menu 113.
fn other_menu(app: &mut App, ui: &mut egui::Ui, who: u32, is_primary: bool) {
    let slot = app.slot;
    let mut act = |ui: &mut egui::Ui, label: &str, c: Cmd| {
        if ui.button(tr(label)).clicked() {
            if !matches!(c, Cmd::ToPrimary(_)) {
                app.pending.push(Cmd::SelectSlot(slot, Some(who)));
            }
            app.pending.push(c);
            ui.close();
        }
    };
    if !is_primary {
        act(ui, "Move to Primary", Cmd::ToPrimary(who));
    }
    act(ui, "Edit", Cmd::Edit);
    if !is_primary {
        act(ui, "Unlink...", Cmd::Unlink);
    }
    act(ui, "Delete...", Cmd::DeletePerson);
    act(ui, "Notes", Cmd::Notes);
    act(ui, "Multimedia", Cmd::Media);
    ui.separator();
    act(ui, "Edit Marriage", Cmd::EditMarriage);
    act(ui, "Edit Parents' Marriage", Cmd::ParentsMarriage);
    act(ui, "Edit Parent Links", Cmd::EditParents);
    ui.separator();
    act(ui, "Add New Individual...", Cmd::AddIndividual);
    act(ui, "Add New Parents...", Cmd::AddOtherParents);
    act(ui, "Add Spouse...", Cmd::AddSpouse);
    act(ui, "Add Child...", Cmd::AddChild);
    ui.separator();
    act(ui, "Reports...", Cmd::Reports);
}

/// Clicks on a box: click highlights, double-click edits or adds, Ctrl or Cmd double-click moves.
fn box_clicks(app: &mut App, ui: &egui::Ui, resp: &egui::Response, who: Option<u32>, slot: Slot, add: Cmd) {
    if resp.clicked() || resp.secondary_clicked() {
        app.pending.push(Cmd::SelectSlot(slot, who));
    }
    if resp.double_clicked() {
        match who {
            Some(r) => {
                let mods = ui.input(|i| i.modifiers);
                if mods.ctrl || mods.command {
                    app.pending.push(Cmd::ToPrimary(r));
                } else {
                    app.pending.push(Cmd::SelectSlot(slot, Some(r)));
                    app.pending.push(Cmd::Edit);
                }
            }
            None => {
                app.pending.push(Cmd::SelectSlot(slot, None));
                app.pending.push(add);
            }
        }
    }
}

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
    let stroke = Stroke::new(1.0, INK);

    // Connecting lines between the primary person and the parents.
    let (jx, top, mid, bottom) = (fit.p(497.0, 0.0).x.round(), fit.p(0.0, 199.0).y.round(), fit.p(0.0, 244.0).y.round(), fit.p(0.0, 310.0).y.round());
    painter.hline(fit.p(463.0, 0.0).x.round()..=jx, mid, stroke);
    painter.vline(jx, top..=bottom, stroke);
    painter.hline(jx..=fit.p(532.0, 0.0).x.round(), top, stroke);
    painter.hline(jx..=fit.p(532.0, 0.0).x.round(), bottom, stroke);

    // Labels, at the positions in the original's drawing code (see the mechanics sheet).
    painter.text(fit.p(539.0, 144.0), Align2::LEFT_CENTER, tr("Parents"), label_font.clone(), INK);
    painter.text(fit.p(49.0, 343.0), Align2::LEFT_CENTER, tr("Spouse"), label_font.clone(), INK);
    painter.text(fit.p(49.0, 458.0), Align2::LEFT_CENTER, tr("Children"), label_font.clone(), INK);
    painter.text(fit.p(614.0, 458.0), Align2::LEFT_CENTER, tr("Born"), label_font.clone(), INK);
    painter.text(fit.p(736.0, 458.0), Align2::LEFT_CENTER, tr("Parent Link"), label_font.clone(), INK);
    // Only drawn when LDS data is in use.
    if app.db.prefs.use_lds {
        painter.text(fit.p(878.0, 458.0), Align2::LEFT_CENTER, tr("Ordinances"), label_font.clone(), INK);
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
    painter.rect_filled(r, 0.0, if m_sel { SELECT } else { Color32::WHITE });
    painter.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
    let ink = if m_sel { Color32::WHITE } else { INK };
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
    let label_w = ui.fonts_mut(|f| f.layout_no_wrap(format!("{} ", tr("Marriage:")), mfont.clone(), INK).size().x);
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
        if ui.put(rect, egui::Button::new(format!("Other marriages ({}/{count})", at + 1)).small()).clicked() {
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
        if ui.put(rect, egui::Button::new(format!("Other parents ({})", links.len())).small()).clicked() {
            app.pending.push(Cmd::ShowParents(next));
        }
    }

    // Children.
    let r = fit.r(42.0, 472.0, 980.0, 697.0);
    painter.rect_filled(r, 0.0, Color32::WHITE);
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
                let ink = if sel { Color32::WHITE } else { INK };
                if sel {
                    ui.painter().rect_filled(row, 0.0, SELECT);
                }
                let Some(p) = app.db.person(c) else { continue };
                let cy = row.center().y;
                let solid = !p.spouses.is_empty();
                let font = plain(14.0);
                ui.painter().text(pos2(row.left() + 20.0, cy), Align2::LEFT_CENTER, format!("{}", i + 1), font.clone(), ink);
                let name_area = Rect::from_min_max(pos2(row.left() + 40.0, row.top()), pos2(x_born - 8.0, row.bottom()));
                ui.painter().text(pos2(name_area.left(), cy), Align2::LEFT_CENTER, clip(ui, &shown_name(app, p), &font, name_area.width() - 12.0), font.clone(), ink);
                draw_marks(ui.painter(), name_area, p, ink, if sel { SELECT } else { Color32::WHITE });
                ui.painter().text(pos2(x_born, cy), Align2::LEFT_CENTER, date_with_star(&p.birth, &p.chr), font.clone(), ink);
                // The column stays blank for an ordinary birth link.
                let rel = mrin.and_then(|m| p.parents.iter().find(|l| l.mrin == m)).map(|l| match l.relation.as_str() {
                    "Biological" | "Unknown" => String::new(),
                    "LDS Sealing" => "Sealing".to_string(),
                    other => other.to_string(),
                }).unwrap_or_default();
                ui.painter().text(pos2(x_link, cy), Align2::LEFT_CENTER, tr(&rel), font.clone(), ink);
                if app.db.prefs.use_lds {
                    ui.painter().text(pos2(x_ord, cy), Align2::LEFT_CENTER, calc::ordinance_flags(&app.db, c), FontId::monospace(13.0), ink);
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
    let stroke = Stroke::new(1.0, INK);
    let font = plain((16.0 * fit.sy.min(fit.sx)).clamp(12.0, 22.0));
    for (x, label) in [(28.0, "1st"), (95.0, "2nd"), (208.0, "3rd"), (395.0, "4th"), (704.0, "5th")] {
        painter.text(fit.p(x, 133.0), Align2::LEFT_CENTER, tr(label), font.clone(), INK);
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
        p.rect_filled(r, 0.0, if sel { SELECT } else { Color32::WHITE });
        p.rect_stroke(r, 0.0, stroke, StrokeKind::Inside);
        if let Some(person) = who.and_then(|w| app.db.person(w)) {
            let ink = if sel { Color32::WHITE } else { INK };
            draw_marks(p, r, person, ink, if sel { SELECT } else { Color32::WHITE });
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
                let galley = ui.fonts_mut(|f| f.layout(text, plain(12.0), INK, 360.0));
                let top = if r.bottom() + galley.size().y + 12.0 > area.bottom() { r.top() - galley.size().y - 10.0 } else { r.bottom() + 1.0 };
                let left = r.left().min(area.right() - galley.size().x - 14.0);
                let bg = Rect::from_min_size(pos2(left, top), galley.size() + vec2(10.0, 8.0));
                egui::Area::new(ui.scope_id().with("infobox")).order(egui::Order::Foreground).fixed_pos(bg.min).interactable(false).show(ui.ctx(), |ui| {
                    let (rect, _) = ui.allocate_exact_size(bg.size(), Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(255, 255, 225));
                    ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, INK), StrokeKind::Inside);
                    ui.painter().galley(rect.min + vec2(5.0, 4.0), galley.clone(), INK);
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
                out.push_str(&format!("\nSpouse: {name}  {}", mar.event.date.trim()));
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
    ui.painter().rect_filled(area, 0.0, Color32::WHITE);
    ui.painter().rect_stroke(area, 0.0, Stroke::new(1.0, INK), StrokeKind::Inside);
    let columns = app.individual.columns.clone();
    let total: f32 = columns.iter().map(|c| column_width(c)).sum::<f32>().max(area.width() - 2.0);
    egui::ScrollArea::horizontal().auto_shrink([false, false]).show(ui, |ui| {
        // Header row. Click sorts by RIN or by name, right-click picks the columns.
        let (head, _) = ui.allocate_exact_size(vec2(total, 22.0), Sense::hover());
        ui.painter().rect_filled(head, 0.0, BG);
        ui.painter().hline(head.x_range(), head.bottom(), Stroke::new(1.0, SHADOW));
        let mut x = head.left();
        for col in &columns {
            let w = column_width(col);
            let r = Rect::from_min_size(pos2(x, head.top()), vec2(w, head.height()));
            let resp = ui.interact(r, ui.scope_id().with(format!("col{col}")), Sense::click());
            ui.painter().vline(r.right() - 1.0, r.y_range(), Stroke::new(1.0, SHADOW));
            let sorted = (col == "RIN" && app.individual.sort_by_rin) || (col == "Full Name" && !app.individual.sort_by_rin);
            let text = if sorted { format!("{} ▾", tr(col)) } else { tr(col).to_string() };
            let head_font = bold(12.0);
            ui.painter().text(pos2(x + 5.0, r.center().y), Align2::LEFT_CENTER, clip(ui, &text, &head_font, w - 8.0), head_font, INK);
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
                    ui.painter().rect_filled(row, 0.0, SELECT);
                    if want {
                        resp.scroll_to_me(None);
                    }
                }
                let ink = if sel { Color32::WHITE } else { INK };
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

impl App {
    pub fn family_view(&mut self, ui: &mut egui::Ui) {
        family(self, ui);
    }

    pub fn pedigree_view(&mut self, ui: &mut egui::Ui) {
        pedigree(self, ui);
    }

    pub fn individual_view(&mut self, ui: &mut egui::Ui) {
        individual(self, ui);
    }

    /// Keys that move around the screen. F, M and C move to the father, mother or a child.
    pub fn view_keys(&mut self, ctx: &egui::Context) {
        let plain_key = |k: Key| ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(k));
        if self.view != View::Individual {
            if plain_key(Key::F) {
                self.pending.push(Cmd::GoFather);
            }
            if plain_key(Key::M) {
                self.pending.push(Cmd::GoMother);
            }
            if plain_key(Key::C) {
                self.pending.push(Cmd::NextChild);
            }
        }
        match self.view {
            View::Family => family_keys(self, ctx),
            View::Pedigree => pedigree_keys(self, ctx),
            View::Individual => individual_keys(self, ctx),
        }
    }

    /// Three flat tabs with a dark divider on the right, measured from the pictures.
    pub fn tabs(&mut self, ui: &mut egui::Ui) {
        let r = ui.max_rect();
        let p = ui.painter().clone();
        p.hline(r.left()..=r.right(), r.bottom() - 1.0, Stroke::new(1.0, SHADOW));
        for (v, label, x0, x1) in [(View::Family, "Family", 8.0, 109.0), (View::Pedigree, "Pedigree", 109.0, 204.0), (View::Individual, "Individual", 204.0, 302.0)] {
            let rect = Rect::from_min_max(pos2(r.left() + x0, r.top() + 1.0), pos2(r.left() + x1, r.bottom()));
            let resp = ui.allocate_rect(rect, Sense::click());
            if self.view == v {
                p.hline(rect.left()..=rect.right(), rect.bottom() - 1.0, Stroke::new(1.0, BG));
                p.vline(rect.left(), rect.y_range(), Stroke::new(1.0, Color32::WHITE));
            }
            p.vline(rect.right(), (rect.top() + 2.0)..=(rect.bottom() - 2.0), Stroke::new(1.0, INK));
            p.text(pos2((x0 + x1) / 2.0 + r.left(), rect.center().y), Align2::CENTER_CENTER, tr(label), plain(15.0), INK);
            if resp.clicked() {
                self.pending.push(Cmd::SetView(v));
            }
        }
    }
}

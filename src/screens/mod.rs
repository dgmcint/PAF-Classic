//! The three main screens and what they share. Positions are the pixels measured from the
//! person's pictures of the original at 1024 by 742; they scale with the window as the original's screens do.

mod family;
mod individual;
mod pedigree;

pub use family::*;
pub use individual::*;
pub use pedigree::*;

use crate::i18n::tr;
use crate::app::*;
use crate::data::*;
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
    let w = |s: &str| ui.fonts_mut(|f| f.layout_no_wrap(s.to_string(), font.clone(), pal().ink).size().x);
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
    let back = if sel { pal().select } else { pal().field };
    painter.rect_filled(rect, 0.0, back);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, pal().ink), StrokeKind::Inside);
    let Some(p) = who.and_then(|r| app.db.person(r)) else { return resp };
    let ink = if sel { pal().select_ink } else { pal().field_ink };
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
    let fill = if solid { pal().ink } else { pal().shadow };
    ui.painter().add(egui::Shape::convex_polygon(vec![a, b, c], fill, Stroke::new(1.0, pal().ink)));
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
        p.hline(r.left()..=r.right(), r.bottom() - 1.0, Stroke::new(1.0, pal().shadow));
        for (v, label, x0, x1) in [(View::Family, "Family", 8.0, 109.0), (View::Pedigree, "Pedigree", 109.0, 204.0), (View::Individual, "Individual", 204.0, 302.0)] {
            let rect = Rect::from_min_max(pos2(r.left() + x0, r.top() + 1.0), pos2(r.left() + x1, r.bottom()));
            let resp = ui.allocate_rect(rect, Sense::click());
            if self.view == v {
                p.hline(rect.left()..=rect.right(), rect.bottom() - 1.0, Stroke::new(1.0, pal().bg));
                p.vline(rect.left(), rect.y_range(), Stroke::new(1.0, pal().light));
            }
            p.vline(rect.right(), (rect.top() + 2.0)..=(rect.bottom() - 2.0), Stroke::new(1.0, pal().ink));
            p.text(pos2((x0 + x1) / 2.0 + r.left(), rect.center().y), Align2::CENTER_CENTER, tr(label), plain(15.0), pal().ink);
            if resp.clicked() {
                self.pending.push(Cmd::SetView(v));
            }
        }
    }
}

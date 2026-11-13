//! The records a family file holds.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
    #[default]
    Unknown,
}

impl Sex {
    pub fn letter(self) -> &'static str {
        match self {
            Sex::Male => "M",
            Sex::Female => "F",
            Sex::Unknown => "U",
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Citation {
    pub source: u32,
    #[serde(default)]
    pub page: String,
    #[serde(default)]
    pub actual_text: String,
    #[serde(default)]
    pub comments: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub place: String,
    #[serde(default)]
    pub citations: Vec<Citation>,
}

impl Event {
    pub fn is_empty(&self) -> bool {
        self.date.trim().is_empty() && self.place.trim().is_empty() && self.citations.is_empty()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ordinance {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub temple: String,
}

impl Ordinance {
    pub fn is_empty(&self) -> bool {
        self.date.trim().is_empty() && self.temple.trim().is_empty()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomEvent {
    pub kind: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub place: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
}

impl Contact {
    pub fn is_empty(&self) -> bool {
        self.address.trim().is_empty() && self.phone.trim().is_empty() && self.email.trim().is_empty()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Media {
    pub file: String,
    #[serde(default)]
    pub caption: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub in_scrapbook: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParentLink {
    pub mrin: u32,
    /// Biological, Adopted, Guardian, Challenged, Disproved, LDS Sealing or Unknown.
    #[serde(default = "default_rel")]
    pub relation: String,
    #[serde(default)]
    pub sealed: Ordinance,
}

pub(crate) fn default_rel() -> String {
    "Biological".into()
}

pub const PARENT_RELATIONS: [&str; 7] =
    ["Biological", "Adopted", "Guardian", "Challenged", "Disproved", "LDS Sealing", "Unknown"];

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Person {
    pub rin: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub aka: String,
    #[serde(default)]
    pub sex: Sex,
    #[serde(default)]
    pub birth: Event,
    #[serde(default)]
    pub chr: Event,
    #[serde(default)]
    pub death: Event,
    #[serde(default)]
    pub burial: Event,
    #[serde(default)]
    pub baptism: Ordinance,
    #[serde(default)]
    pub endowment: Ordinance,
    #[serde(default)]
    pub afn: String,
    #[serde(default)]
    pub custom_id: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub contact: Contact,
    #[serde(default)]
    pub media: Vec<Media>,
    #[serde(default)]
    pub custom: Vec<CustomEvent>,
    #[serde(default)]
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub spouses: Vec<u32>,
    #[serde(default)]
    pub parents: Vec<ParentLink>,
    #[serde(default)]
    pub changed: String,
    /// The record number this person had before Renumber Records (kept as REFN, type "Old ID").
    #[serde(default)]
    pub old_id: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Marriage {
    pub mrin: u32,
    #[serde(default)]
    pub husband: Option<u32>,
    #[serde(default)]
    pub wife: Option<u32>,
    #[serde(default)]
    pub children: Vec<u32>,
    #[serde(default)]
    pub event: Event,
    #[serde(default)]
    pub divorced: bool,
    #[serde(default)]
    pub sealing: Ordinance,
    #[serde(default)]
    pub custom: Vec<CustomEvent>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub citations: Vec<Citation>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Source {
    pub id: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub publication: String,
    #[serde(default)]
    pub abbreviation: String,
    #[serde(default)]
    pub repository: Option<u32>,
    #[serde(default)]
    pub comments: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub id: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub contact: Contact,
}

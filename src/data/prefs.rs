//! Preferences saved with each family file.

use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Prefs {
    #[serde(default = "yes")]
    pub use_lds: bool,
    #[serde(default = "yes")]
    pub auto_slash: bool,
    #[serde(default)]
    pub show_relationships: bool,
    #[serde(default = "yes")]
    pub edit_marriage_when_created: bool,
    #[serde(default = "yes")]
    pub use_list_navigating: bool,
    #[serde(default)]
    pub verify_new_places: bool,
    #[serde(default)]
    pub day_first_numeric: bool,
    #[serde(default)]
    pub prepared_by: Contact,
    #[serde(default)]
    pub prepared_name: String,
    #[serde(default)]
    pub log_changes: bool,
    #[serde(default = "backup_default")]
    pub backup_frequency: u32,
    #[serde(default)]
    pub closes_since_backup: u32,
    #[serde(default)]
    pub changed_since_backup: bool,
    /// Show each person's number after their name on the screens. Off is an estimate:
    /// the original keeps this per file and its default wasn't found.
    #[serde(default)]
    pub show_rin: bool,
    /// 0 none, 1 dates only, 2 dates and family information.
    #[serde(default = "floating_default")]
    pub floating_info: u8,
    #[serde(default = "locked_default")]
    pub locked_info: u8,
    #[serde(default = "wait_default")]
    pub info_wait_tenths: u32,
    #[serde(default = "yes")]
    pub show_lds_reports: bool,
    #[serde(default = "yes")]
    pub check_duplicates: bool,
    #[serde(default = "yes")]
    pub use_fathers_surname: bool,
    #[serde(default)]
    pub enter_as_tab: bool,
    #[serde(default)]
    pub allow_afn_edit: bool,
    #[serde(default = "yes")]
    pub shade_reports: bool,
    #[serde(default = "age_low")]
    pub mother_low: i32,
    #[serde(default = "age_high")]
    pub mother_high: i32,
    #[serde(default = "age_low")]
    pub father_low: i32,
    #[serde(default = "age_high")]
    pub father_high: i32,
    #[serde(default = "span_default")]
    pub spouse_span: i32,
    #[serde(default = "yes")]
    pub check_lds: bool,
    #[serde(default = "yes")]
    pub check_order: bool,
    #[serde(default = "merge_years_default")]
    pub merge_years: i32,
    #[serde(default = "yes")]
    pub merge_notes: bool,
    #[serde(default = "yes")]
    pub merge_citations: bool,
    #[serde(default = "yes")]
    pub merge_media: bool,
    #[serde(default = "yes")]
    pub merge_confirm: bool,
}

fn yes() -> bool {
    true
}

fn backup_default() -> u32 {
    1
}

fn floating_default() -> u8 {
    1
}

fn locked_default() -> u8 {
    2
}

fn wait_default() -> u32 {
    10
}

fn age_low() -> i32 {
    16
}

fn age_high() -> i32 {
    45
}

fn span_default() -> i32 {
    15
}

fn merge_years_default() -> i32 {
    10
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            use_lds: true,
            auto_slash: true,
            show_relationships: false,
            edit_marriage_when_created: true,
            use_list_navigating: true,
            verify_new_places: false,
            day_first_numeric: false,
            prepared_by: Contact::default(),
            prepared_name: String::new(),
            log_changes: false,
            backup_frequency: backup_default(),
            closes_since_backup: 0,
            changed_since_backup: false,
            show_rin: false,
            floating_info: floating_default(),
            locked_info: locked_default(),
            info_wait_tenths: wait_default(),
            show_lds_reports: true,
            check_duplicates: true,
            use_fathers_surname: true,
            enter_as_tab: false,
            allow_afn_edit: false,
            shade_reports: true,
            mother_low: age_low(),
            mother_high: age_high(),
            father_low: age_low(),
            father_high: age_high(),
            spouse_span: span_default(),
            check_lds: true,
            check_order: true,
            merge_years: merge_years_default(),
            merge_notes: true,
            merge_citations: true,
            merge_media: true,
            merge_confirm: true,
        }
    }
}

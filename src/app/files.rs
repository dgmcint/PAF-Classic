//! Opening, saving, importing and backing up family files, and where settings are kept.

use crate::i18n::trf;
use super::*;
use crate::formats::gedcom;
use crate::data::*;
use crate::formats::paf;
use std::path::{Path, PathBuf};

/// Opens a file or web address with whatever this computer uses for it.
pub fn open_with_system(target: impl AsRef<std::ffi::OsStr>) -> std::io::Result<()> {
    let target = target.as_ref();
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("rundll32");
        c.arg("url.dll,FileProtocolHandler");
        c
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut cmd = std::process::Command::new("xdg-open");
    cmd.arg(target).spawn().map(|_| ())
}

/// Runs a file reader so that a fault inside it becomes a message rather than closing the app.
pub fn guarded<T>(read: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(read)).unwrap_or_else(|_| {
        Err("This file could not be read. It may use a layout this program doesn't handle yet.".into())
    })
}

pub fn config_dir() -> PathBuf {
    #[allow(unused_variables)]
    let home = || std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    #[cfg(target_os = "macos")]
    let base = home().join("Library/Application Support");
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
    });
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let base = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config"));
    base.join(APP_NAME)
}

pub(crate) fn recent_file() -> PathBuf {
    config_dir().join("recent.json")
}

impl App {
    /// A copy of the family file as it is now, saved beside it, before a big change.
    pub fn safety_copy(&mut self, why: &str) -> Option<PathBuf> {
        let path = self.path.clone()?;
        let stem = path.file_stem()?.to_str()?.to_string();
        let t = crate::rules::today();
        let copy = path.with_file_name(format!("{stem} before {why} {:04}-{:02}-{:02}.{FILE_EXT}", t.0, t.1, t.2));
        let bytes = serde_json::to_vec_pretty(&self.db).ok()?;
        std::fs::write(&copy, bytes).ok()?;
        Some(copy)
    }

    pub(crate) fn save_now(&mut self) -> bool {
        let Some(path) = self.path.clone() else { return false };
        match serde_json::to_vec_pretty(&self.db) {
            Ok(bytes) => {
                // Write beside the file, then swap, so a crash cannot leave half a file.
                let tmp = path.with_extension("pafc.saving");
                if let Err(e) = std::fs::write(&tmp, bytes).and_then(|_| std::fs::rename(&tmp, &path)) {
                    self.message("Could not save", trf("Could not save {file}:\n{error}", &[("file", &path.display()), ("error", &e)]));
                    return false;
                }
                self.dirty = false;
                self.remember_recent(&path);
                true
            }
            Err(e) => {
                self.message("Could not save", e.to_string());
                false
            }
        }
    }

    fn remember_recent(&mut self, path: &Path) {
        // The automated tests save scratch files, which must never join the person's list.
        if cfg!(test) || self.recent.first().map(|p| p == path).unwrap_or(false) {
            return;
        }
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_path_buf());
        self.recent.truncate(4);
        let _ = std::fs::create_dir_all(config_dir());
        if let Ok(b) = serde_json::to_vec(&self.recent) {
            let _ = std::fs::write(recent_file(), b);
        }
    }

    pub(crate) fn pick_family_filter() -> rfd::FileDialog {
        rfd::FileDialog::new()
            .add_filter("Family files", &[FILE_EXT, "paf", "ged"])
            .add_filter("All files", &["*"])
    }

    pub fn open_path(&mut self, path: PathBuf) {
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.message("Could not open", trf("Could not open {file}:\n{error}", &[("file", &path.display()), ("error", &e)]));
                return;
            }
        };
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        match ext.as_str() {
            "paf" => match guarded(|| paf::import(&bytes)) {
                Ok(got) => {
                    self.load_db(got.db, None);
                    self.dirty = true;
                    self.message("Family file read", format!("{}\n\nUse File, Save As to keep this as a {APP_NAME} family file.", got.summary));
                }
                Err(e) => self.message("Could not read this .paf file", e),
            },
            "ged" => self.import_gedcom(&bytes),
            _ => match serde_json::from_slice::<Db>(&bytes) {
                Ok(db) => {
                    self.load_db(db, Some(path.clone()));
                    self.remember_recent(&path);
                }
                Err(_) => {
                    if paf::identify(&bytes) != paf::Kind::Other {
                        match guarded(|| paf::import(&bytes)) {
                            Ok(got) => {
                                self.load_db(got.db, None);
                                self.dirty = true;
                                self.message("Family file read", got.summary);
                            }
                            Err(e) => self.message("Could not read this .paf file", e),
                        }
                    } else if bytes.starts_with(b"0 HEAD") || bytes.windows(6).take(16).any(|w| w == b"0 HEAD") {
                        self.import_gedcom(&bytes);
                    } else {
                        self.message("Could not open", "This file is not a valid family file. It may be corrupt.");
                    }
                }
            },
        }
    }

    fn import_gedcom(&mut self, bytes: &[u8]) {
        match guarded(|| gedcom::import(bytes)) {
            Ok((db, report)) => {
                self.load_db(db, None);
                self.dirty = true;
                self.message("GEDCOM import", format!("{}\n\nUse File, Save As to keep this as a {APP_NAME} family file.", report.summary()));
            }
            Err(e) => self.message("GEDCOM import", e),
        }
    }

    /// The guide's way to combine files: Import adds to the open family, and an empty family
    /// simply fills up. The records get the next free numbers, so nothing already here changes.
    pub(crate) fn import_into_current(&mut self, path: &Path) {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                self.message("Could not open", trf("Could not open {file}:\n{error}", &[("file", &path.display()), ("error", &e)]));
                return;
            }
        };
        let is_paf = path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("paf"));
        let got = guarded(|| if is_paf { paf::import(&bytes).map(|g| (g.db, g.summary)) } else { gedcom::import(&bytes).map(|(d, r)| (d, r.summary())) });
        match got {
            Ok((db, summary)) => {
                self.db.absorb(db);
                self.touch();
                self.selected = self.primary;
                self.message("Import", summary);
            }
            Err(e) => self.message("Import", e),
        }
    }

    pub(crate) fn save_as_dialog(&mut self) {
        let mut d = rfd::FileDialog::new().add_filter("Family files", &[FILE_EXT]).set_file_name(format!("Family.{FILE_EXT}"));
        if let Some(p) = &self.path {
            if let Some(folder) = p.parent() {
                d = d.set_directory(folder);
            }
        }
        if let Some(mut p) = d.save_file() {
            if p.extension().is_none() {
                p.set_extension(FILE_EXT);
            }
            self.path = Some(p);
            self.save_now();
        }
    }

    pub(crate) fn write_backup(&mut self) {
        let name = self.path.as_ref().and_then(|p| p.file_stem()).and_then(|s| s.to_str()).unwrap_or("Family").to_string();
        let t = crate::rules::today();
        let file = format!("{name} backup {:04}-{:02}-{:02}.{FILE_EXT}", t.0, t.1, t.2);
        if let Some(p) = rfd::FileDialog::new().add_filter("Family files", &[FILE_EXT]).set_file_name(file).save_file() {
            match serde_json::to_vec_pretty(&self.db).map_err(|e| e.to_string()).and_then(|b| std::fs::write(&p, b).map_err(|e| e.to_string())) {
                Ok(()) => {
                    self.db.prefs.closes_since_backup = 0;
                    self.touch();
                    self.db.prefs.changed_since_backup = false;
                    if self.path.is_some() {
                        self.save_now();
                    }
                    self.message("Backup", trf("Backup saved to {file}.", &[("file", &p.display())]));
                }
                Err(e) => self.message("Backup", trf("The backup could not be saved:\n{error}", &[("error", &e)])),
            }
        }
    }
}

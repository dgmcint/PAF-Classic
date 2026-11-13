//! GEDCOM 5.5 import and export.

mod read;
mod text;
mod write;

pub use read::*;
pub use text::*;
pub use write::*;


fn event_label(tag: &str) -> &'static str {
    match tag {
        "OCCU" => "Occupation",
        "RELI" => "Religion",
        "EDUC" => "Education",
        "NATI" => "Nationality",
        "CAST" => "Caste",
        "PROP" => "Property",
        "CENS" => "Census",
        "IMMI" => "Immigration",
        "EMIG" => "Emigration",
        "NATU" => "Naturalization",
        "GRAD" => "Graduation",
        "RETI" => "Retirement",
        "PROB" => "Probate",
        "WILL" => "Will",
        "ADOP" => "Adoption",
        "BARM" => "Bar Mitzvah",
        "BASM" => "Bas Mitzvah",
        "BLES" => "Blessing",
        "CHRA" => "Adult Christening",
        "CONF" => "Confirmation",
        "FCOM" => "First Communion",
        "ORDN" => "Ordination",
        "DSCR" => "Physical Description",
        "IDNO" => "National ID",
        "NCHI" => "Number of Children",
        "NMR" => "Number of Marriages",
        "SSN" => "Social Security Number",
        "ENGA" => "Engagement",
        "MARB" => "Marriage Banns",
        "MARC" => "Marriage Contract",
        "MARL" => "Marriage License",
        "MARS" => "Marriage Settlement",
        "ANUL" => "Annulment",
        "DIVF" => "Divorce Filed",
        _ => "Event",
    }
}

fn tag_for_label(label: &str) -> Option<&'static str> {
    const TAGS: [&str; 34] = [
        "OCCU", "RELI", "EDUC", "NATI", "CAST", "PROP", "CENS", "IMMI", "EMIG", "NATU", "GRAD", "RETI", "PROB", "WILL", "ADOP", "BARM", "BASM", "BLES",
        "CHRA", "CONF", "FCOM", "ORDN", "DSCR", "IDNO", "NCHI", "NMR", "SSN", "ENGA", "MARB", "MARC", "MARL", "MARS", "ANUL", "DIVF",
    ];
    TAGS.into_iter().find(|t| event_label(t) == label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::*;

    #[test]
    fn export_then_import_keeps_everything() {
        let mut db = sample_family();
        let me = db.home.unwrap();
        db.sources.insert(1, Source { id: 1, title: "Parish register".into(), repository: Some(1), ..Default::default() });
        db.repositories.insert(1, Repository { id: 1, name: "County archive".into(), contact: Contact::default() });
        {
            let p = db.people.get_mut(&me).unwrap();
            p.birth.citations.push(Citation { source: 1, page: "p. 12".into(), ..Default::default() });
            p.notes = "First line\nSecond line ".to_string() + &"long ".repeat(100);
            p.custom.push(CustomEvent { kind: "Occupation".into(), date: "1950".into(), place: "Boise".into(), description: "Teacher".into() });
            p.custom.push(CustomEvent { kind: "Elected mayor".into(), date: "1961".into(), place: String::new(), description: "Town hall".into() });
            p.baptism = Ordinance { date: "1 Jan 1930".into(), temple: "SLAKE".into() };
            p.parents[0].relation = "Adopted".into();
            p.parents[0].sealed = Ordinance { date: "2 Feb 1931".into(), temple: "LOGAN".into() };
            p.media.push(Media { file: "photo.jpg".into(), caption: "Portrait".into(), ..Default::default() });
        }
        let text = export(&db, &ExportOptions::default(), "PAF Classic");
        let (back, report) = import(text.as_bytes()).unwrap();
        assert_eq!(report.people, db.people.len());
        assert_eq!(report.marriages, db.marriages.len());
        assert!(report.skipped_tags.is_empty(), "{:?}", report.skipped_tags);
        assert_eq!(back.marriages, db.marriages);
        for (rin, a) in &db.people {
            let b = &back.people[rin];
            assert_eq!(a.name, b.name);
            assert_eq!(a.sex, b.sex);
            assert_eq!(a.birth, b.birth);
            assert_eq!(a.death, b.death);
            assert_eq!(a.spouses, b.spouses);
            assert_eq!(a.parents, b.parents);
            assert_eq!(a.notes, b.notes);
            assert_eq!(a.custom, b.custom);
            assert_eq!(a.media, b.media);
        }
        assert_eq!(back.sources, db.sources);
    }

    #[test]
    fn reads_other_programs_files() {
        let text = "0 HEAD\n1 CHAR ANSEL\n0 @P1@ INDI\n1 NAME Ann /Lee/\n1 SEX F\n1 BIRT\n2 DATE 1 JAN 1900\n1 FAMC @X@\n1 _ODD thing\n0 @P2@ INDI\n1 NAME Joe /Lee/\n0 @X@ FAM\n1 HUSB @P2@\n1 CHIL @P1@\n0 TRLR\n";
        let (db, r) = import(text.as_bytes()).unwrap();
        assert_eq!(db.people.len(), 2);
        assert_eq!(db.father(1), Some(2));
        // Joe's FAMS line was missing, and the import mends it.
        assert_eq!(db.person(2).unwrap().spouses.len(), 1);
        assert_eq!(r.skipped_tags.get("_ODD"), Some(&1));
    }

    #[test]
    fn ansel_accents() {
        let bytes = [b'J', 0xE2, b'o', b's', 0xE2, b'e'];
        assert_eq!(ansel_to_text(&bytes), "Jósé");
    }

    #[test]
    fn privacy_hides_living() {
        let db = sample_family();
        let opts = ExportOptions { full_living: false, names_on_living: false, ..Default::default() };
        let text = export(&db, &opts, "PAF Classic");
        assert!(text.contains("1 NAME LIVING"));
        assert!(!text.contains("Susan"));
        assert!(text.contains("Thomas Edward"));
    }

    #[test]
    fn rejects_incomplete() {
        assert!(import(b"0 HEAD\n").is_err());
        assert!(import(b"hello\n").is_err());
    }
}

#[cfg(test)]
mod home_tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::data::*;

    #[test]
    fn home_person_comes_from_the_file_and_survives_export() {
        // Website-style IDs: numbered 1, 2... in file order, and the first person is home.
        let text = "0 HEAD\n1 CHAR UTF-8\n0 @I900000001@ INDI\n1 NAME Ann /Lee/\n1 FAMC @F777777@\n0 @I12345678@ INDI\n1 NAME Joe /Lee/\n1 FAMS @F777777@\n0 @F777777@ FAM\n1 HUSB @I12345678@\n1 CHIL @I900000001@\n0 TRLR\n";
        let (mut db, _) = import(text.as_bytes()).unwrap();
        assert_eq!(db.people.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(db.marriages.keys().copied().collect::<Vec<_>>(), vec![1]);
        assert_eq!(db.home, Some(1));
        db.home = Some(2);
        db.renumber(&[2, 1]);
        let out = export(&db, &ExportOptions::default(), "PAF Classic").replace("\r\n", "\n");
        assert!(out.contains("1 _HOME @I1@"), "{out}");
        assert!(out.contains("1 REFN 2\n2 TYPE Old ID"), "{out}");
        let (again, _) = import(out.as_bytes()).unwrap();
        assert_eq!(again.name(again.home.unwrap()), db.name(db.home.unwrap()));
        assert_eq!(again.people[&1].old_id, "2");
        // PAF-style numbers are kept, and without a header mark the first person is home.
        let paf = "0 HEAD\n0 @I7@ INDI\n1 NAME A /B/\n0 @I3@ INDI\n1 NAME C /D/\n0 TRLR\n";
        let (db, _) = import(paf.as_bytes()).unwrap();
        assert_eq!(db.people.keys().copied().collect::<Vec<_>>(), vec![3, 7]);
        assert_eq!(db.home, Some(7));
    }
}

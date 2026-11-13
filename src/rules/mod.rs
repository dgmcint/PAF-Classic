//! Rules taken from the original program: Soundex, dates, relationships, record checks.

mod case;
mod checks;
mod dates;
mod ordinances;
mod relationships;
mod search;
mod soundex;

pub use case::*;
pub use checks::*;
pub use dates::*;
pub use ordinances::*;
pub use relationships::*;
pub use search::*;
pub use soundex::*;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::*;

    #[test]
    fn soundex_matches_the_guide() {
        for n in ["Smith", "Smithe", "Smyth", "Smythe"] {
            assert_eq!(soundex(n), "S530");
        }
        assert_eq!(soundex("Kelly"), "K400");
        assert_eq!(soundex("McCarthy"), "M263");
        assert_eq!(soundex("Mc Carthy"), "M263");
    }

    #[test]
    fn soundex_keeps_the_original_quirks() {
        // First letter's own code counts, so the F is dropped.
        assert_eq!(soundex("Pfister"), "P236");
        // H separates repeats in this program (the census rule would give A261).
        assert_eq!(soundex("Ashcraft"), "A226");
    }

    #[test]
    fn dates_parse() {
        let d = parse_date("23 Mar 1742", false).unwrap();
        assert_eq!((d.year, d.month, d.day), (1742, 3, 23));
        assert_eq!(parse_date("Dec 1845", false).unwrap().month, 12);
        assert_eq!(parse_date("1799", false).unwrap().year, 1799);
        assert_eq!(parse_date("Abt 1850", false).unwrap().qualifier, Qual::About);
        assert_eq!(parse_date("14/15 May 1854", false).unwrap().day, 14);
        assert_eq!(parse_date("28 Apr 1721/1722", false).unwrap().year, 1721);
        assert_eq!(parse_date("<1854>", false).unwrap().year, 1854);
        assert_eq!(parse_date("From 10 Sep 1867 To 17 Aug 1869", false).unwrap().year, 1867);
        assert_eq!(parse_date("10 Sep 1867 - 17 Aug 1869", false).unwrap().year, 1867);
        assert_eq!(parse_date("Abt <1854>", false).unwrap().year, 1854);
        assert_eq!(parse_date("<1854>", false).unwrap().year, 1854);
        assert_eq!(parse_date("BC 44", false).unwrap().year, -44);
        assert_eq!(parse_date("44 BC", false).unwrap().year, -44);
        assert_eq!(parse_date("44 BC", false).unwrap().year, -44);
        let us = parse_date("3/11/1955", false).unwrap();
        assert_eq!((us.month, us.day), (3, 11));
        let eu = parse_date("3/11/1955", true).unwrap();
        assert_eq!((eu.month, eu.day), (11, 3));
        assert!(parse_date("feast of St John", false).is_none());
        assert!(is_standard_date("Stillborn", false));
        assert!(!is_standard_date("feast of St John", false));
    }

    #[test]
    fn date_math() {
        assert_eq!(from_serial(serial(1900, 3, 1)), (1900, 3, 1));
        assert_eq!(date_difference((2000, 1, 31), (2000, 3, 1)), (0, 1, 1));
        assert_eq!(date_difference((1950, 6, 15), (2000, 6, 15)), (50, 0, 0));
        assert_eq!(date_add((2000, 1, 31), 0, 1, 0), (2000, 2, 29));
        assert_eq!(date_add((1999, 12, 31), 0, 0, 1), (2000, 1, 1));
    }

    #[test]
    fn relationship_words() {
        assert_eq!(relationship_word(Sex::Male, 0, 1), "Father");
        assert_eq!(relationship_word(Sex::Female, 0, 2), "Grandmother");
        assert_eq!(relationship_word(Sex::Male, 0, 3), "Great Grandfather");
        assert_eq!(relationship_word(Sex::Male, 0, 4), "2nd Great Grandfather");
        assert_eq!(relationship_word(Sex::Female, 1, 0), "Daughter");
        assert_eq!(relationship_word(Sex::Female, 1, 1), "Sister");
        assert_eq!(relationship_word(Sex::Male, 1, 2), "Uncle");
        assert_eq!(relationship_word(Sex::Male, 1, 4), "Great Grand Uncle");
        assert_eq!(relationship_word(Sex::Male, 1, 5), "2nd Great Grand Uncle");
        assert_eq!(relationship_word(Sex::Female, 2, 1), "Niece");
        assert_eq!(relationship_word(Sex::Male, 2, 2), "1st Cousin");
        assert_eq!(relationship_word(Sex::Male, 3, 2), "1st Cousin 1 Time Removed");
        assert_eq!(relationship_word(Sex::Male, 3, 3), "2nd Cousin");
        assert_eq!(relationship_word(Sex::Male, 5, 3), "2nd Cousin 2 Times Removed");
    }

    #[test]
    fn relationships_in_sample() {
        let db = sample_family();
        let me = db.home.unwrap();
        let dad = db.father(me).unwrap();
        assert_eq!(relationships(&db, dad, me)[0].words, "Father");
        let wife = db.person(me).unwrap().spouses[0];
        let wife = db.spouse_in(wife, me).unwrap();
        assert!(relationships(&db, wife, me)[0].words.starts_with("Wife of"));
        // Parents who share two common ancestors only print one word.
        let kids = db.children_of(me);
        let r = relationships(&db, kids[0], kids[1]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].words, "Brother");
        assert_eq!(r[0].ancestors.len(), 2);
    }

    #[test]
    fn checks_catch_problems() {
        let mut db = sample_family();
        let me = db.home.unwrap();
        assert!(check_person(&db, me).is_empty(), "{:?}", check_person(&db, me));
        db.people.get_mut(&me).unwrap().death.date = "1900".into();
        let p = check_person(&db, me);
        assert!(p.iter().any(|m| m.contains("later than death date")));
    }

    #[test]
    fn case_changes() {
        assert_eq!(mixed_case("SMITH", false), "Smith");
        assert_eq!(mixed_case("LA ROSE", false), "La Rose");
        assert_eq!(mixed_case("VOM RATH", false), "vom Rath");
        assert_eq!(mixed_case("ISLE OF THE LAKE, USA", true), "Isle of the Lake, USA");
        assert_eq!(mixed_case("MCDONALD", false), "McDonald");
        assert_eq!(mixed_case("MACKINTOSH", false), "MacKintosh");
        assert_eq!(mixed_case("VAN DER BERG", false), "van der Berg");
        // The original's rules: all capitals only, numerals, joined words and separators.
        assert_eq!(mixed_case("McDonald", false), "McDonald");
        assert_eq!(mixed_case("JOHN /SMITH/ III", false), "John /Smith/ III");
        assert_eq!(mixed_case("O'BRIEN", false), "O'Brien");
        assert_eq!(mixed_case("FITZGERALD", false), "FitzGerald");
        assert_eq!(mixed_case("DELL'ARTE", false), "dell'Arte");
        assert_eq!(mixed_case("SMITH-JONES", false), "Smith-Jones");
        assert_eq!(mixed_case("BOISE, ADA, IDAHO, USA", true), "Boise, Ada, Idaho, USA");
        assert_eq!(mixed_case("DE SOTO, FLORIDA", true), "De Soto, Florida");
    }

    #[test]
    fn global_replace_works() {
        let mut db = sample_family();
        let n = global_replace(&mut db, "Places", "Boise", "Nampa", false, true);
        assert!(n > 0);
        assert!(db.all_places().iter().all(|p| !p.contains("Boise")));
    }

    #[test]
    fn duplicates_found() {
        let mut db = sample_family();
        let mut twin = db.person(db.home.unwrap()).unwrap().clone();
        twin.rin = 0;
        twin.spouses.clear();
        twin.parents.clear();
        twin.name = "Thomas Edward /Harow/".into();
        let t = db.add_person(twin);
        let c = duplicate_candidates(&db, true, 2);
        assert!(c.contains(&(db.home.unwrap(), t)));
        assert_eq!(with_surname_marks("John Henry Smith Jr."), "John Henry /Smith/ Jr.");
        assert_eq!(with_surname_marks("John Smith"), "John /Smith/");
        assert_eq!(with_surname_marks("Madonna"), "Madonna //");
        // Two words stay a given name and surname, even when the last looks like an ending.
        assert_eq!(with_surname_marks("Anna V"), "/Anna/ V");
        assert_eq!(with_surname_marks("Ludwig van Beethoven"), "Ludwig /van Beethoven/");
        assert_eq!(with_surname_marks("Maria Elena de la Cruz"), "Maria Elena /de la Cruz/");
        assert_eq!(with_surname_marks("Hans van der Berg Jr."), "Hans /van der Berg/ Jr.");
        assert_eq!(with_surname_marks("Angus Mac Donald"), "Angus /Mac Donald/");
        assert_eq!(with_surname_marks("Van Gogh"), "Van /Gogh/");
        db.merge_people(db.home.unwrap(), t);
        assert!(db.person(t).is_none());
    }
}

//! The made-up sample family under Help.

use super::*;

/// A small made-up family used for the Help menu's sample and for tests.
pub fn sample_family() -> Db {
    let mut db = Db::default();
    let add = |db: &mut Db, name: &str, sex: Sex, b: (&str, &str), d: (&str, &str)| {
        let mut p = Person { name: name.into(), sex, ..Default::default() };
        p.birth.date = b.0.into();
        p.birth.place = b.1.into();
        p.death.date = d.0.into();
        p.death.place = d.1.into();
        db.add_person(p)
    };
    let me = add(&mut db, "Thomas Edward /Harrow/", Sex::Male, ("14 Mar 1921", "Boise, Ada, Idaho, USA"), ("2 Oct 1998", "Boise, Ada, Idaho, USA"));
    let wife = add(&mut db, "Clara Louise /Whitcomb/", Sex::Female, ("8 Jun 1924", "Ogden, Weber, Utah, USA"), ("30 Jan 2003", "Boise, Ada, Idaho, USA"));
    let dad = add(&mut db, "Edward James /Harrow/", Sex::Male, ("Abt 1890", "York, Yorkshire, England"), ("12 Feb 1960", "Boise, Ada, Idaho, USA"));
    let mum = add(&mut db, "Ida May /Larkin/", Sex::Female, ("3 Sep 1894", "Provo, Utah, Utah, USA"), ("7 Jul 1971", "Boise, Ada, Idaho, USA"));
    let gdad = add(&mut db, "William /Harrow/", Sex::Male, ("1862", "York, Yorkshire, England"), ("1931", "York, Yorkshire, England"));
    let gmum = add(&mut db, "Hannah /Booth/", Sex::Female, ("1865", "Leeds, Yorkshire, England"), ("1940", "York, Yorkshire, England"));
    let c1 = add(&mut db, "Robert Thomas /Harrow/", Sex::Male, ("22 Apr 1947", "Boise, Ada, Idaho, USA"), ("", ""));
    let c2 = add(&mut db, "Margaret Ann /Harrow/", Sex::Female, ("5 Nov 1949", "Boise, Ada, Idaho, USA"), ("", ""));
    let c3 = add(&mut db, "Paul Edward /Harrow/", Sex::Male, ("19 Aug 1953", "Boise, Ada, Idaho, USA"), ("", ""));
    let gc = add(&mut db, "Susan /Harrow/", Sex::Female, ("11 Jan 1975", "Boise, Ada, Idaho, USA"), ("", ""));
    let inlaw = add(&mut db, "Janet /Moore/", Sex::Female, ("1948", "Nampa, Canyon, Idaho, USA"), ("", ""));
    let m1 = db.new_marriage_for(me);
    db.link_spouse_to(m1, wife);
    db.marriages.get_mut(&m1).unwrap().event = Event { date: "18 May 1946".into(), place: "Boise, Ada, Idaho, USA".into(), citations: vec![] };
    for c in [c1, c2, c3] {
        db.add_child(m1, c);
    }
    let m2 = db.new_marriage_for(dad);
    db.link_spouse_to(m2, mum);
    db.marriages.get_mut(&m2).unwrap().event = Event { date: "2 Jun 1918".into(), place: "Provo, Utah, Utah, USA".into(), citations: vec![] };
    db.add_child(m2, me);
    let m3 = db.new_marriage_for(gdad);
    db.link_spouse_to(m3, gmum);
    db.add_child(m3, dad);
    let m4 = db.new_marriage_for(c1);
    db.link_spouse_to(m4, inlaw);
    db.add_child(m4, gc);
    db.people.get_mut(&me).unwrap().notes = "Sample family made up for trying out the program.".into();
    db.home = Some(me);
    db
}

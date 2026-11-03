# PAF Classic — Feature status

How PAF Classic compares with the features described in the *Personal Ancestral
File 5.0 User's Guide*. Checked against the program's code in November 2026
(version 0.1.x).

**Key:** ✅ built in · 🟡 partly built · ❌ not built yet · ➖ left out on purpose

"Built in" means the feature exists in the program. Not every feature has been
tested by hand yet, so please [report anything that doesn't behave as expected](https://github.com/dgmcint/PAF-Classic/issues).

---

## Managing family files

| Feature | Status | Notes |
| --- | --- | --- |
| Create, open and close a family file | ✅ | Saves as `.pafc` |
| Open PAF 5 `.paf` files | 🟡 | People, families, dates, places and notes come in. Sources, citations, custom events and pictures don't yet. |
| Save back into a `.paf` file | ➖ | PAF Classic saves its own `.pafc` files. The original `.paf` file is never changed. |
| Search the computer for `.paf` files | ❌ | Use File → Open and browse instead. |
| Convert older PAF files (versions 3 and 4) | ✅ | Read when opened |
| Delete a family file | ✅ | |
| Backup and Restore | ✅ | |
| Check/Repair | ✅ | |
| Split a file | ✅ | Export a focus list as GEDCOM |
| Combine files | ✅ | Import adds to the open file |
| File properties | ✅ | |

## Screens and navigation

| Feature | Status | Notes |
| --- | --- | --- |
| Family, Pedigree and Individual screens | ✅ | |
| InfoBoxes on the Pedigree screen | ✅ | |
| Move a person to the primary position | ✅ | |
| Keyboard shortcuts | ✅ | Help → Keyboard lists them |
| Individual screen: add, remove, reorder and sort columns | ✅ | |
| Individual screen: resize columns | ❌ | Columns have fixed widths |
| Character map and number-pad diacritics | ➖ | Use the computer's own accent and emoji/character tools |

## Adding and editing people

| Feature | Status | Notes |
| --- | --- | --- |
| Add individuals, families, spouses, children and parents | ✅ | |
| Add an individual not linked to a family | ✅ | |
| Unknown spouses | ✅ | |
| Ditto (copy from selected person, father, mother or primary) | ✅ | |
| Cut, copy and paste in fields | ✅ | Standard system shortcuts |
| Multiple spouses, and showing another spouse | ✅ | |
| Multiple sets of parents (birth, adopted, step and others) | ✅ | |
| Choose primary parents, change the parent link | ✅ | |
| Sealing-to-parents information | ✅ | |
| Edit individual and marriage records | ✅ | |
| Order children, spouses and parents | ✅ | |
| Remove from a family, unlink from parents | ✅ | |
| Delete an individual or a marriage | ✅ | |
| Custom events and attributes | ✅ | Including the Individual and Marriage Event Lists |
| Templates, with Phonetic and Romanized name fields | ❌ | |

## Sources, repositories, notes and contacts

| Feature | Status | Notes |
| --- | --- | --- |
| Sources and citations on events | ✅ | |
| Source List: add, edit, delete | ✅ | |
| Repositories: add, edit, remove, delete | ✅ | |
| Multimedia on sources and citations | ❌ | |
| Notes: add, edit, delete | ✅ | |
| Find and replace in notes | ✅ | |
| Note tags and the Notes Selector | ❌ | |
| Contact information | ✅ | |

## Multimedia

| Feature | Status | Notes |
| --- | --- | --- |
| Add, edit and remove pictures and files for a person | ✅ | |
| Captions and descriptions | ✅ | |
| Mark items for the scrapbook | ✅ | |
| Edit a photo (crop, rotate) | ❌ | |

## Finding people

| Feature | Status | Notes |
| --- | --- | --- |
| Home person | ✅ | |
| Individual list and Marriage list | ✅ | |
| Find by RIN or MRIN | ✅ | |
| Descendancy list | ✅ | |
| Advanced Focus/Filter: relationship and field filters | ✅ | |
| Save and reuse focus lists | ✅ | |

## GEDCOM

| Feature | Status | Notes |
| --- | --- | --- |
| Import GEDCOM | ✅ | |
| Export GEDCOM, all or part of a file | ✅ | |
| Export options: notes, sources, multimedia, contacts, LDS data, living people | ✅ | |
| Ancestral File / Pedigree Resource File submission | ➖ | These services have closed |
| TempleReady export | ➖ | TempleReady has been retired; its toolbar button became Backup |

## Reports and printing

| Feature | Status | Notes |
| --- | --- | --- |
| Pedigree charts, including blank ones | ✅ | |
| Family group records, including blank ones | ✅ | |
| Ancestry and descendancy charts | ✅ | |
| Books | ✅ | |
| Individual summaries | ✅ | |
| Lists | ✅ | |
| Calendars | ✅ | |
| Scrapbook | 🟡 | Lists captions; doesn't print the pictures yet |
| Custom reports | 🟡 | Can be built and printed, but not saved and reused |
| Preview and save as PDF | ✅ | Print from the PDF viewer |
| Page setup, printer choice, report fonts | ❌ | Handled by the PDF viewer when printing |
| Photos on reports | ❌ | |
| Change a report's title | ✅ | |
| Reports in languages other than English | ❌ | Printed reports are in English |

## Tools

| Feature | Status | Notes |
| --- | --- | --- |
| Create web pages | ✅ | |
| Match/Merge by name or AFN | ✅ | |
| Match/Merge by record serial number | ❌ | |
| Merge duplicate sources | ✅ | |
| Merge duplicate repositories | ❌ | |
| Global search and replace | ✅ | |
| Change names to mixed case | ✅ | |
| Relationship, Date and Soundex calculators | ✅ | |
| Internet searches (FamilySearch, Ancestry, Genealogy.com, RootsWeb) | ✅ | |
| Notepad | ✅ | |

## Preferences

| Tab | Status |
| --- | --- |
| General, File, Names, Prepared By, InfoBox, Formats | ✅ |
| Fonts | ❌ |
| Multimedia | ❌ |
| Folders | ❌ |
| Templates | ❌ |

---

## Planned next

The gaps that matter most to day-to-day family history, roughly in order:

1. Bring sources, citations, custom events and pictures in from `.paf` files
2. Print photos on reports, and print the scrapbook with its pictures
3. Multimedia on sources and citations
4. Note tags and the Notes Selector
5. Save and reuse custom reports
6. Resizable columns on the Individual screen
7. Merge duplicate repositories, and Match/Merge by record serial number
8. Templates, and the Fonts, Multimedia and Folders preference tabs
9. Photo editing

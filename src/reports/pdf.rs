//! A small PDF writer for the reports.

use super::*;

/// Latin-1 text for the built-in PDF fonts, with PDF's special characters escaped.
fn pdf_text(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for c in s.chars() {
        let code = c as u32;
        let byte = match c {
            '(' | ')' | '\\' => {
                out.push(b'\\');
                code as u8
            }
            '…' => 0x85,
            '–' => 0x96,
            '—' => 0x97,
            '’' => 0x92,
            _ if code < 0x20 => b' ',
            _ if code <= 0xFF => code as u8,
            _ => b'?',
        };
        out.push(byte);
    }
    out
}

/// Writes a report as PDF. It uses PDF's built-in fonts, so letters outside Western European
/// alphabets print as a question mark.
pub fn to_pdf(r: &Report) -> Vec<u8> {
    let mut objs: Vec<Vec<u8>> = Vec::new();
    let n = r.pages.len().max(1);
    let content_start = 5;
    let page_start = content_start + n;
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", page_start + i)).collect();
    objs.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objs.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), n).into_bytes());
    objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec());
    objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec());
    let empty = Page::default();
    for i in 0..n {
        let page = r.pages.get(i).unwrap_or(&empty);
        let mut c: Vec<u8> = Vec::new();
        c.extend_from_slice(b"0.5 w\n");
        for op in &page.ops {
            match op {
                Op::Text { x, y, size, bold, text } => {
                    let font = if *bold { "F2" } else { "F1" };
                    c.extend_from_slice(format!("BT /{font} {size} Tf {x:.1} {:.1} Td (", PAGE_H - y - size).as_bytes());
                    c.extend(pdf_text(text));
                    c.extend_from_slice(b") Tj ET\n");
                }
                Op::Line { x1, y1, x2, y2 } => {
                    c.extend_from_slice(format!("{x1:.1} {:.1} m {x2:.1} {:.1} l S\n", PAGE_H - y1, PAGE_H - y2).as_bytes());
                }
                Op::Rect { x, y, w, h, shade } => {
                    let op = if *shade { "0.9 g" } else { "1 g" };
                    c.extend_from_slice(format!("{op} {x:.1} {:.1} {w:.1} {h:.1} re B 0 g\n", PAGE_H - y - h).as_bytes());
                }
            }
        }
        let mut obj = format!("<< /Length {} >>\nstream\n", c.len()).into_bytes();
        obj.extend(c);
        obj.extend_from_slice(b"\nendstream");
        objs.push(obj);
    }
    for i in 0..n {
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
                content_start + i
            )
            .into_bytes(),
        );
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in &offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes());
    out
}

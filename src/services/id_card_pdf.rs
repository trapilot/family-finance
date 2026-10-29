// Packs CCCD photos into one A4 PDF: one row per person (front | back) at real
// card size, 5 people per page, in the order given. Hand-written PDF so the
// JPEGs are embedded as-is (DCTDecode) — no re-encoding, no extra crates.

use crate::error::{AppError, Result};

/// One person's photos; a missing side leaves its slot empty
pub struct CardPair {
    pub front: Option<Vec<u8>>,
    pub back:  Option<Vec<u8>>,
}

const PT_PER_MM: f64 = 72.0 / 25.4;
const PAGE_W:    f64 = 210.0;   // A4, mm
const PAGE_H:    f64 = 297.0;
const CARD_W:    f64 = 85.6;    // ISO/IEC 7810 ID-1, same as CCCD
const CARD_H:    f64 = 53.98;
const COL_GAP:   f64 = 6.0;
const ROW_GAP:   f64 = 2.5;
pub const ROWS_PER_PAGE: usize = 5;

pub fn build(people: &[CardPair]) -> Result<Vec<u8>> {
    if people.is_empty() {
        return Err(AppError::Validation("No ID cards selected".into()));
    }

    // Every photo becomes one image XObject, in reading order
    let mut images = Vec::new();
    for p in people {
        for jpeg in [&p.front, &p.back].into_iter().flatten() {
            images.push((jpeg.as_slice(), jpeg_info(jpeg)?));
        }
    }

    let pages: Vec<&[CardPair]> = people.chunks(ROWS_PER_PAGE).collect();

    // Object ids: 1 catalog, 2 page tree, then images, then (page, content) per page
    let first_image = 3;
    let first_page  = first_image + images.len();
    let page_id     = |i: usize| first_page + i * 2;

    let mut pdf = Pdf::new();

    pdf.object(1, b"<< /Type /Catalog /Pages 2 0 R >>");
    let kids: Vec<String> = (0..pages.len()).map(|i| format!("{} 0 R", page_id(i))).collect();
    pdf.object(2, format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), pages.len()).as_bytes());

    for (i, (jpeg, info)) in images.iter().enumerate() {
        let color_space = if info.components == 1 { "/DeviceGray" } else { "/DeviceRGB" };
        let dict = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace {} /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>",
            info.width, info.height, color_space, jpeg.len()
        );
        pdf.stream(first_image + i, &dict, jpeg);
    }

    // Rows centred vertically on the page, columns centred horizontally
    let block_h = |rows: usize| rows as f64 * CARD_H + (rows.saturating_sub(1)) as f64 * ROW_GAP;
    let left    = (PAGE_W - (2.0 * CARD_W + COL_GAP)) / 2.0;
    let top     = (PAGE_H - block_h(ROWS_PER_PAGE)) / 2.0;

    let mut next_image = 0;
    for (pi, rows) in pages.iter().enumerate() {
        let mut content = String::new();
        let mut resources = Vec::new();

        for (ri, person) in rows.iter().enumerate() {
            // PDF y grows upwards from the bottom of the page
            let y = PAGE_H - top - ri as f64 * (CARD_H + ROW_GAP) - CARD_H;
            for (ci, side) in [&person.front, &person.back].into_iter().enumerate() {
                if side.is_none() { continue; }
                let info = &images[next_image].1;
                let x = left + ci as f64 * (CARD_W + COL_GAP);
                let name = format!("Im{next_image}");
                content.push_str(&place_image(&name, info, x, y));
                resources.push(format!("/{name} {} 0 R", first_image + next_image));
                next_image += 1;
            }
        }

        let page = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /XObject << {} >> >> /Contents {} 0 R >>",
            PAGE_W * PT_PER_MM, PAGE_H * PT_PER_MM, resources.join(" "), page_id(pi) + 1
        );
        pdf.object(page_id(pi), page.as_bytes());
        pdf.stream(page_id(pi) + 1, &format!("<< /Length {} >>", content.len()), content.as_bytes());
    }

    Ok(pdf.finish(first_page + pages.len() * 2))
}

/// Draw command fitting the photo inside one card slot, keeping its aspect ratio
fn place_image(name: &str, info: &JpegInfo, x: f64, y: f64) -> String {
    let scale = (CARD_W / info.width as f64).min(CARD_H / info.height as f64);
    let (w, h) = (info.width as f64 * scale, info.height as f64 * scale);
    let (x, y) = (x + (CARD_W - w) / 2.0, y + (CARD_H - h) / 2.0);
    format!(
        "q {:.2} 0 0 {:.2} {:.2} {:.2} cm /{name} Do Q\n",
        w * PT_PER_MM, h * PT_PER_MM, x * PT_PER_MM, y * PT_PER_MM
    )
}

// ─── Minimal PDF writer ───────────────────────────────────────────────────────

struct Pdf {
    buf:     Vec<u8>,
    offsets: Vec<(usize, usize)>,  // (object id, byte offset)
}

impl Pdf {
    fn new() -> Self {
        // Binary comment marks the file as binary for transfer tools
        Self { buf: b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec(), offsets: Vec::new() }
    }

    fn object(&mut self, id: usize, body: &[u8]) {
        self.offsets.push((id, self.buf.len()));
        self.buf.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        self.buf.extend_from_slice(body);
        self.buf.extend_from_slice(b"\nendobj\n");
    }

    fn stream(&mut self, id: usize, dict: &str, data: &[u8]) {
        let mut body = Vec::with_capacity(dict.len() + data.len() + 32);
        body.extend_from_slice(dict.as_bytes());
        body.extend_from_slice(b"\nstream\n");
        body.extend_from_slice(data);
        body.extend_from_slice(b"\nendstream");
        self.object(id, &body);
    }

    fn finish(mut self, object_count: usize) -> Vec<u8> {
        self.offsets.sort();
        let xref_at = self.buf.len();
        let mut xref = format!("xref\n0 {}\n0000000000 65535 f \n", object_count);
        for (_, offset) in &self.offsets {
            xref.push_str(&format!("{offset:010} 00000 n \n"));
        }
        xref.push_str(&format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
            object_count
        ));
        self.buf.extend_from_slice(xref.as_bytes());
        self.buf
    }
}

// ─── JPEG header ──────────────────────────────────────────────────────────────

struct JpegInfo {
    width:      u32,
    height:     u32,
    components: u8,
}

/// Read size and colour components from the JPEG's start-of-frame marker
fn jpeg_info(data: &[u8]) -> Result<JpegInfo> {
    let bad = || AppError::Parse("ID card photo is not a valid JPEG".into());
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err(bad());
    }
    let mut i = 2;
    while i + 4 <= data.len() {
        if data[i] != 0xFF { return Err(bad()); }
        let marker = data[i + 1];
        if marker == 0xFF { i += 1; continue; }  // fill byte
        let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        // SOF0..SOF15, except DHT (C4), JPG (C8) and DAC (CC)
        if (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker) {
            let sof = data.get(i + 4..i + 10).ok_or_else(bad)?;
            return Ok(JpegInfo {
                height:     u16::from_be_bytes([sof[1], sof[2]]) as u32,
                width:      u16::from_be_bytes([sof[3], sof[4]]) as u32,
                components: sof[5],
            });
        }
        i += 2 + len;
    }
    Err(bad())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Just enough of a JPEG for the header parser: SOI, APP0, SOF0
    fn fake_jpeg(w: u16, h: u16) -> Vec<u8> {
        let mut v = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0x00, 0x00, 0xFF, 0xC0, 0x00, 0x11, 0x08];
        v.extend_from_slice(&h.to_be_bytes());
        v.extend_from_slice(&w.to_be_bytes());
        v.extend_from_slice(&[3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xD9]);
        v
    }

    #[test]
    fn reads_jpeg_size() {
        let info = jpeg_info(&fake_jpeg(1280, 806)).unwrap();
        assert_eq!((info.width, info.height, info.components), (1280, 806, 3));
        assert!(jpeg_info(b"not a jpeg").is_err());
    }

    #[test]
    fn five_people_per_page_and_valid_xref() {
        let people: Vec<CardPair> = (0..7).map(|i| CardPair {
            front: Some(fake_jpeg(1280, 806)),
            back:  if i == 3 { None } else { Some(fake_jpeg(806, 1280)) },
        }).collect();
        let pdf = build(&people).unwrap();
        let text = String::from_utf8_lossy(&pdf);

        assert!(text.contains("/Count 2"));
        assert_eq!(text.matches("/Subtype /Image").count(), 13);
        assert_eq!(text.matches(" Do Q").count(), 13);

        // startxref points at the xref table, and each entry at its "N 0 obj"
        let start: usize = text.rsplit("startxref\n").next().unwrap().lines().next().unwrap().parse().unwrap();
        assert!(pdf[start..].starts_with(b"xref"));
        let table = String::from_utf8_lossy(&pdf[start..]);
        for (id, line) in table.lines().skip(3).take_while(|l| l.ends_with(" n ")).enumerate() {
            let offset: usize = line[..10].parse().unwrap();
            assert!(pdf[offset..].starts_with(format!("{} 0 obj", id + 1).as_bytes()));
        }
    }

    #[test]
    fn empty_selection_is_an_error() {
        assert!(build(&[]).is_err());
    }
}

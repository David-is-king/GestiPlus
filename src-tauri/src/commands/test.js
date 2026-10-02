use crate::models::{SaleItem, StoreSettings};
use printpdf::*;
use std::fs::File;
use std::io::{BufWriter, Cursor};

// Image des icônes (sac de ciment, tuyau, robinet, brouette, ventilateur, fers à béton, truelle).
// Chemin relatif à CE fichier .rs : place l'image dans src-tauri/assets/ (si ce fichier est dans src-tauri/src/).
const HEADER_ICONS: &[u8] = include_bytes!("../assets/entete_icones.png");
const ICONS_RATIO: f32 = 248.0 / 556.0; // hauteur / largeur de l'image

pub struct InvoiceData {
    pub invoice_number: String,
    pub date: String,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub items: Vec<SaleItem>,
    pub total: f64,
    pub is_proforma: bool,
}

pub fn generate_invoice_pdf(
    settings: &StoreSettings,
    invoice: &InvoiceData,
    output_path: &std::path::Path,
) -> Result<(), String> {
    let (doc, page1, layer1) = PdfDocument::new(
        &format!("Facture {}", invoice.invoice_number),
        Mm(210.0),
        Mm(297.0),
        "Contenu",
    );
    let layer = doc.get_page(page1).get_layer(layer1);

    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| e.to_string())?;

    // --- PALETTE DE COULEURS DU MODÈLE ---
    let blue_primary = Color::Rgb(Rgb::new(0.0, 0.44, 0.75, None)); // Bleu vif (#0070C0)
    let blue_light = Color::Rgb(Rgb::new(0.85, 0.91, 0.96, None)); // Bleu très clair pour le total
    let gray_border = Color::Rgb(Rgb::new(0.80, 0.80, 0.80, None)); // Lignes du tableau
    let dark_text = Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None));
    let white = Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None));

    // ==========================================
    // 1. EN-TÊTE GAUCHE : nom, slogan, adresse, nom commercial, téléphones
    // ==========================================
    let (lx1, lx2, ly1, ly2) = (14.0_f32, 128.0_f32, 231.0_f32, 283.0_f32);
    draw_rounded_rect(&layer, lx1, ly1, lx2, ly2, 4.5, blue_primary.clone(), 2.2);
    let cx = (lx1 + lx2) / 2.0;
    let max_w = lx2 - lx1 - 10.0;

    let clean = |o: &Option<String>| -> Option<String> {
        o.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(String::from)
    };

    let mut addr_parts: Vec<String> = Vec::new();
    if let Some(v) = clean(&settings.quartier) { addr_parts.push(v); }
    if let Some(v) = clean(&settings.rue) { addr_parts.push(v); }
    if let Some(v) = clean(&settings.emplacement) { addr_parts.push(v); }
    let address = addr_parts.join(", ");

    let mut phones: Vec<String> = Vec::new();
    for p in [&settings.phone1, &settings.phone2, &settings.phone3] {
        if let Some(v) = clean(p) { phones.push(v); }
    }
    let phones_line = if phones.is_empty() { String::new() } else { format!("Tél : {}", phones.join(" / ")) };
    let email_line = clean(&settings.email).map(|e| format!("Email : {}", e)).unwrap_or_default();

    let name_text = settings.name.trim().to_string();
    let slogan = clean(&settings.slogan).unwrap_or_else(|| "Quincaillerie generale".to_string());
    let commercial = clean(&settings.commercial_name);

    // Hauteur d'une ligne (mm) pour une taille de police (pt)
    let line_h = |size: f32| size * 0.3528 * 1.2;

    // Hauteur des lignes de taille fixe
    let mut fixed_h = line_h(13.0) + 3.0; // slogan + espace
    if !address.is_empty() { fixed_h += line_h(9.5); }
    if commercial.is_some() { fixed_h += 3.0; }
    if !phones_line.is_empty() { fixed_h += line_h(10.5); }
    if !email_line.is_empty() { fixed_h += line_h(9.0); }

    // Taille commune « nom de la boutique » = « nom commercial » : la plus grande qui tienne
    // (2 lignes maximum chacun) dans le cadre.
    let mut big = 26.0_f32;
    loop {
        let n = wrap_text(&name_text, big, max_w).len();
        let c = commercial.as_ref().map(|t| wrap_text(t, big, max_w).len()).unwrap_or(0);
        let h = (n + c) as f32 * line_h(big) + fixed_h;
        if (n <= 2 && c <= 2 && h <= (ly2 - ly1) - 8.0) || big <= 12.0 {
            break;
        }
        big -= 1.0;
    }

    let name_lines = wrap_text(&name_text, big, max_w);
    let comm_lines = commercial.as_ref().map(|t| wrap_text(t, big, max_w)).unwrap_or_default();
    let total_h = (name_lines.len() + comm_lines.len()) as f32 * line_h(big) + fixed_h;
    let mut top = ly2 - ((ly2 - ly1) - total_h) / 2.0; // bloc centré verticalement dans le cadre

    layer.set_fill_color(blue_primary.clone());
    // 1) Nom de la boutique (gros)
    for l in &name_lines {
        put_line(&layer, l, big, cx, &mut top, &font_bold);
    }
    // 2) Slogan
    put_line(&layer, &slogan, 13.0, cx, &mut top, &font_bold);
    top -= 3.0;
    // 3) Emplacement (quartier, rue, emplacement)
    if !address.is_empty() {
        put_line(&layer, &address, 9.5, cx, &mut top, &font_bold);
    }
    // 4) Nom commercial (même taille que le nom de la boutique)
    if !comm_lines.is_empty() {
        top -= 3.0;
        for l in &comm_lines {
            put_line(&layer, l, big, cx, &mut top, &font_bold);
        }
    }
    // 5) Numéros de téléphone (puis email, s'il existe)
    if !phones_line.is_empty() {
        put_line(&layer, &phones_line, 10.5, cx, &mut top, &font_bold);
    }
    if !email_line.is_empty() {
        put_line(&layer, &email_line, 9.0, cx, &mut top, &font_bold);
    }

    // ==========================================
    // 2. EN-TÊTE DROIT : « Bamako, le <date> » + cadre avec icônes
    // ==========================================
    layer.set_fill_color(blue_primary.clone());
    draw_right(
        &layer,
        &format!("Bamako, le {}", format_date(&invoice.date)),
        10.0,
        196.0,
        279.0,
        &font_bold,
    );

    let (rx1, rx2, ry1, ry2) = (135.0_f32, 196.0_f32, ly1, 274.0_f32);
    draw_rounded_rect(&layer, rx1, ry1, rx2, ry2, 4.5, blue_primary.clone(), 2.2);
    let icons_w = (rx2 - rx1) - 4.0;
    let icons_h = icons_w * ICONS_RATIO;
    draw_header_icons(&layer, rx1 + 2.0, ry1 + ((ry2 - ry1) - icons_h) / 2.0, icons_w)?;

    // Ligne « PRO FORMA N° … » (équivalent de « REÇU N° » sur le modèle)
    layer.set_fill_color(blue_primary.clone());
    layer.use_text(
        if invoice.is_proforma { "PRO FORMA N°" } else { "FACTURE N°" },
        12.0,
        Mm(rx1),
        Mm(ly1 - 8.0),
        &font_bold,
    );
    layer.set_fill_color(dark_text.clone());
    draw_right(&layer, &invoice.invoice_number, 11.0, 196.0, ly1 - 8.0, &font_bold);

    // ==========================================
    // 3. BLOCS "FACTURE POUR" ET "EXPÉDIÉ À"
    // ==========================================
    let client_y = 212.0;

    // Bloc "FACTURE POUR :"
    layer.set_fill_color(dark_text.clone());
    draw_polygon(
        &layer,
        vec![
            point(18.0, client_y),
            point(100.0, client_y),
            point(100.0, client_y - 5.0),
            point(18.0, client_y - 5.0),
        ],
        PolygonMode::Fill,
    );
    layer.set_fill_color(white.clone());
    layer.use_text("FACTURE POUR :", 8.0, Mm(20.0), Mm(client_y - 3.8), &font_bold);

    layer.set_fill_color(dark_text.clone());
    let client_name = invoice.customer_name.clone().unwrap_or_else(|| "Client comptoir".into());
    layer.use_text(&client_name, 8.5, Mm(20.0), Mm(client_y - 9.0), &font_bold);
    if let Some(phone) = &invoice.customer_phone {
        if !phone.is_empty() {
            layer.use_text(format!("Tél : {}", phone), 8.0, Mm(20.0), Mm(client_y - 13.0), &font_regular);
        }
    }

    // Bloc "EXPÉDIÉ À :"
    layer.set_fill_color(blue_primary.clone());
    draw_polygon(
        &layer,
        vec![
            point(110.0, client_y),
            point(192.0, client_y),
            point(192.0, client_y - 5.0),
            point(110.0, client_y - 5.0),
        ],
        PolygonMode::Fill,
    );
    layer.set_fill_color(white.clone());
    layer.use_text("EXPÉDIÉ À :", 8.0, Mm(112.0), Mm(client_y - 3.8), &font_bold);

    layer.set_fill_color(dark_text.clone());
    layer.use_text(&client_name, 8.5, Mm(112.0), Mm(client_y - 9.0), &font_bold);

    // ==========================================
    // 4. TABLEAU DES ARTICLES (STYLE EXCEL)
    // ==========================================
    let table_top = 190.0;
    let table_bottom = 85.0;
    let row_height = 5.5;

    // Colonnes (X min et X max)
    let col_desc = 18.0;
    let col_qte = 120.0;
    let col_pu = 142.0;
    let col_montant = 168.0;
    let col_end = 192.0;

    // En-tête bleu du tableau
    layer.set_fill_color(blue_primary.clone());
    draw_polygon(
        &layer,
        vec![
            point(col_desc, table_top),
            point(col_end, table_top),
            point(col_end, table_top - 6.0),
            point(col_desc, table_top - 6.0),
        ],
        PolygonMode::Fill,
    );

    layer.set_fill_color(white.clone());
    layer.use_text("DESCRIPTION", 8.0, Mm(col_desc + 4.0), Mm(table_top - 4.2), &font_bold);
    layer.use_text("QTÉ", 8.0, Mm(col_qte + 4.0), Mm(table_top - 4.2), &font_bold);
    layer.use_text("PRIX UNITAIRE", 8.0, Mm(col_pu + 2.0), Mm(table_top - 4.2), &font_bold);
    layer.use_text("MONTANT", 8.0, Mm(col_montant + 4.0), Mm(table_top - 4.2), &font_bold);

    // Quadrillage complet et remplissage des lignes
    let mut current_y = table_top - 6.0;
    let max_rows = ((table_top - 6.0 - table_bottom) / row_height) as usize;

    for i in 0..max_rows {
        let line_y = current_y - row_height;

        if i < invoice.items.len() {
            let item = &invoice.items[i];
            layer.set_fill_color(dark_text.clone());
            layer.use_text(&item.product_name, 8.0, Mm(col_desc + 2.0), Mm(current_y - 4.0), &font_regular);
            layer.use_text(item.quantity.to_string(), 8.0, Mm(col_qte + 8.0), Mm(current_y - 4.0), &font_regular);
            layer.use_text(format_amount(item.unit_price), 8.0, Mm(col_pu + 10.0), Mm(current_y - 4.0), &font_regular);
            layer.use_text(format_amount(item.subtotal), 8.0, Mm(col_montant + 8.0), Mm(current_y - 4.0), &font_regular);
        } else {
            layer.set_fill_color(Color::Rgb(Rgb::new(0.65, 0.65, 0.65, None)));
            layer.use_text("0,00", 7.5, Mm(col_montant + 12.0), Mm(current_y - 4.0), &font_regular);
        }

        draw_line(&layer, col_desc, line_y, col_end, line_y, gray_border.clone(), 0.15);
        current_y = line_y;
    }

    // Lignes verticales du tableau
    for &x in &[col_desc, col_qte, col_pu, col_montant, col_end] {
        draw_line(&layer, x, table_top, x, table_bottom, gray_border.clone(), 0.15);
    }

    // ==========================================
    // 5. MERCI & BLOC TOTAUX (SOUS-TOTAL, TAXE, TOTAL)
    // ==========================================
    layer.set_fill_color(blue_primary.clone());
    layer.use_text("MERCI", 14.0, Mm(50.0), Mm(60.0), &font_bold);

    let totals_top = 80.0;

    // Sous-total (fond bleu, texte blanc)
    layer.set_fill_color(blue_primary.clone());
    draw_polygon(
        &layer,
        vec![
            point(120.0, totals_top),
            point(165.0, totals_top),
            point(165.0, totals_top - 6.0),
            point(120.0, totals_top - 6.0),
        ],
        PolygonMode::Fill,
    );
    layer.set_fill_color(white.clone());
    layer.use_text("SOUS-TOTAL", 8.0, Mm(124.0), Mm(totals_top - 4.2), &font_bold);
    layer.set_fill_color(dark_text.clone());
    layer.use_text(format_amount(invoice.total), 8.0, Mm(170.0), Mm(totals_top - 4.2), &font_regular);

    // Ligne Taxe
    draw_line(&layer, 120.0, totals_top - 6.0, 192.0, totals_top - 6.0, gray_border.clone(), 0.2);
    layer.use_text("REMISE / TAXE", 8.0, Mm(124.0), Mm(totals_top - 10.2), &font_bold);
    layer.use_text("0,00", 8.0, Mm(175.0), Mm(totals_top - 10.2), &font_regular);

    // Ligne TOTAL (bleu clair)
    layer.set_fill_color(blue_light.clone());
    draw_polygon(
        &layer,
        vec![
            point(120.0, totals_top - 12.0),
            point(192.0, totals_top - 12.0),
            point(192.0, totals_top - 18.0),
            point(120.0, totals_top - 18.0),
        ],
        PolygonMode::Fill,
    );

    layer.set_fill_color(dark_text.clone());
    layer.use_text("TOTAL", 9.0, Mm(124.0), Mm(totals_top - 16.2), &font_bold);
    layer.use_text(
        format!("{} {}", format_amount(invoice.total), settings.currency),
        9.5,
        Mm(162.0),
        Mm(totals_top - 16.2),
        &font_bold,
    );

    // Contour du bloc total
    draw_rect_outline(&layer, 120.0, totals_top, 192.0, totals_top - 18.0, gray_border.clone(), 0.3);

    // ==========================================
    // 6. PIED DE PAGE
    // ==========================================
    layer.set_fill_color(blue_primary);
    layer.use_text(
        "En cas de questions concernant ce devis / cette facture, veuillez nous contacter.",
        7.5,
        Mm(45.0),
        Mm(30.0),
        &font_regular,
    );

    doc.save(&mut BufWriter::new(
        File::create(output_path).map_err(|e| e.to_string())?,
    ))
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn format_amount(v: f64) -> String {
    format!("{:.0}", v)
}

fn point(x: f32, y: f32) -> (Point, bool) {
    (Point::new(Mm(x), Mm(y)), false)
}

fn draw_polygon(layer: &PdfLayerReference, points: Vec<(Point, bool)>, mode: PolygonMode) {
    layer.add_polygon(Polygon {
        rings: vec![points],
        mode,
        winding_order: WindingOrder::NonZero,
    });
}

fn draw_line(layer: &PdfLayerReference, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, thickness: f32) {
    layer.set_outline_color(color);
    layer.set_outline_thickness(thickness * 10.0);
    let line = Line {
        points: vec![point(x1, y1), point(x2, y2)],
        is_closed: false,
    };
    layer.add_line(line);
}

fn draw_rect_outline(layer: &PdfLayerReference, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, thickness: f32) {
    draw_line(layer, x1, y1, x2, y1, color.clone(), thickness);
    draw_line(layer, x2, y1, x2, y2, color.clone(), thickness);
    draw_line(layer, x2, y2, x1, y2, color.clone(), thickness);
    draw_line(layer, x1, y2, x1, y1, color, thickness);
}

/// Rectangle à coins arrondis (contour seulement). `thickness_pt` est en points PDF.
fn draw_rounded_rect(
    layer: &PdfLayerReference,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    r: f32,
    color: Color,
    thickness_pt: f32,
) {
    layer.set_outline_color(color);
    layer.set_outline_thickness(thickness_pt);

    // Centre de chaque coin + angle de départ de l'arc (sens antihoraire)
    let corners = [
        (x2 - r, y2 - r, 0.0_f32),   // haut droit
        (x1 + r, y2 - r, 90.0),      // haut gauche
        (x1 + r, y1 + r, 180.0),     // bas gauche
        (x2 - r, y1 + r, 270.0),     // bas droit
    ];
    let steps = 10;
    let mut pts: Vec<(Point, bool)> = Vec::new();
    for (cx, cy, start) in corners {
        for i in 0..=steps {
            let a = (start + 90.0 * i as f32 / steps as f32).to_radians();
            pts.push(point(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    draw_polygon(layer, pts, PolygonMode::Stroke);
}

/// Insère l'image des icônes ; (x, y) = coin bas-gauche, `width_mm` = largeur voulue.
fn draw_header_icons(layer: &PdfLayerReference, x: f32, y: f32, width_mm: f32) -> Result<(), String> {
    use printpdf::image_crate::codecs::png::PngDecoder;

    let decoder = PngDecoder::new(Cursor::new(HEADER_ICONS)).map_err(|e| e.to_string())?;
    let image = Image::try_from(decoder).map_err(|e| e.to_string())?;

    let dpi = 300.0_f32;
    let natural_w_mm = image.image.width.0 as f32 / dpi * 25.4;
    let scale = width_mm / natural_w_mm;

    image.add_to_layer(
        layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(x)),
            translate_y: Some(Mm(y)),
            scale_x: Some(scale),
            scale_y: Some(scale),
            dpi: Some(dpi),
            ..Default::default()
        },
    );
    Ok(())
}

// ---------- Texte centré / aligné à droite (largeurs Helvetica-Bold) ----------

fn helv_bold_width(c: char) -> u32 {
    const W: [u32; 95] = [
        278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, // espace .. /
        556, 556, 556, 556, 556, 556, 556, 556, 556, 556, // 0-9
        333, 333, 584, 584, 584, 611, 975, // : ; < = > ? @
        722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722,
        667, 611, 722, 667, 944, 667, 667, 611, // A-Z
        333, 278, 333, 584, 556, 333, // [ \ ] ^ _ `
        556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389,
        556, 333, 611, 556, 778, 556, 556, 500, // a-z
        389, 280, 389, 584, // { | } ~
    ];
    match c {
        ' '..='~' => W[c as usize - 32],
        'é' | 'è' | 'ê' | 'à' | 'â' | 'ç' => 556,
        'É' | 'È' => 667,
        'ô' | 'ù' | 'û' => 611,
        '°' => 400,
        _ => 556,
    }
}

fn text_width_mm(text: &str, size_pt: f32) -> f32 {
    let units: u32 = text.chars().map(helv_bold_width).sum();
    units as f32 / 1000.0 * size_pt * 0.3528
}

fn draw_centered(layer: &PdfLayerReference, text: &str, size: f32, cx: f32, y: f32, font: &IndirectFontRef) {
    let w = text_width_mm(text, size);
    layer.use_text(text, size, Mm(cx - w / 2.0), Mm(y), font);
}

fn draw_right(layer: &PdfLayerReference, text: &str, size: f32, x_right: f32, y: f32, font: &IndirectFontRef) {
    let w = text_width_mm(text, size);
    layer.use_text(text, size, Mm(x_right - w), Mm(y), font);
}

/// Écrit une ligne centrée en haut de `top`, puis fait descendre `top` d'une hauteur de ligne.
fn put_line(layer: &PdfLayerReference, text: &str, size: f32, cx: f32, top: &mut f32, font: &IndirectFontRef) {
    draw_centered(layer, text, size, cx, *top - size * 0.3528 * 0.9, font);
    *top -= size * 0.3528 * 1.2;
}

/// Coupe un texte en lignes qui tiennent dans `max_w` mm (retour à la ligne entre les mots).
fn wrap_text(text: &str, size: f32, max_w: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let cand = if cur.is_empty() { word.to_string() } else { format!("{} {}", cur, word) };
        if cur.is_empty() || text_width_mm(&cand, size) <= max_w {
            cur = cand;
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// "2026-09-30" -> "30/09/2026" (si le format n'est pas reconnu, la date est laissée telle quelle).
fn format_date(d: &str) -> String {
    let head: String = d.chars().take(10).collect();
    let parts: Vec<&str> = head.split('-').collect();
    if parts.len() == 3 && parts[0].len() == 4 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
        format!("{}/{}/{}", parts[2], parts[1], parts[0])
    } else {
        d.to_string()
    }
}

let (rx1, rx2, ry1, ry2) = (135.0_f32, 196.0_f32, ly1, 274.0_f32);
let (rx1, rx2, ry1, ry2) = (143.0, 196.0, 247.0, 274.0);
let (lx1, lx2, ly1, ly2) = (14.0, 135.0, 247.0, 283.0);
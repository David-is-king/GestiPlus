use crate::models::{SaleItem, StoreSettings};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

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
    let blue_primary = Color::Rgb(Rgb::new(0.0, 0.44, 0.75, None));  // Bleu vif (#0070C0)
    let blue_light = Color::Rgb(Rgb::new(0.85, 0.91, 0.96, None));    // Bleu très clair pour le total
    let gray_bg = Color::Rgb(Rgb::new(0.94, 0.94, 0.94, None));       // Fond gris clair
    let gray_border = Color::Rgb(Rgb::new(0.80, 0.80, 0.80, None));   // Lignes du tableau
    let dark_text = Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None));
    let white = Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None));

    // ==========================================
    // 1. EN-TÊTE GAUCHE (Logo + Infos Entreprise)
    // ==========================================
    draw_tool_icon(&layer, 18.0, 272.0, blue_primary.clone());

    layer.set_fill_color(blue_primary.clone());
    layer.use_text(&settings.name, 11.0, Mm(18.0), Mm(260.0), &font_bold);

    layer.set_fill_color(dark_text.clone());
    let mut y = 255.0;

    let mut addr_parts: Vec<String> = Vec::new();
    if let Some(v) = &settings.quartier { if !v.is_empty() { addr_parts.push(v.clone()); } }
    if let Some(v) = &settings.rue { if !v.is_empty() { addr_parts.push(v.clone()); } }
    if let Some(v) = &settings.emplacement { if !v.is_empty() { addr_parts.push(v.clone()); } }
    if !addr_parts.is_empty() {
        layer.use_text(addr_parts.join(", "), 8.0, Mm(18.0), Mm(y), &font_regular);
        y -= 4.0;
    }

    let mut phones: Vec<String> = Vec::new();
    for p in [&settings.phone1, &settings.phone2, &settings.phone3] {
        if let Some(v) = p { if !v.is_empty() { phones.push(v.clone()); } }
    }
    if !phones.is_empty() {
        layer.use_text(format!("Tél : {}", phones.join(" / ")), 8.0, Mm(18.0), Mm(y), &font_regular);
        y -= 4.0;
    }

    if let Some(e) = &settings.email {
        if !e.is_empty() {
            layer.use_text(format!("Email : {}", e), 8.0, Mm(18.0), Mm(y), &font_regular);
        }
    }

    // ==========================================
    // 2. EN-TÊTE DROIT (Titre + Tableau Métadonnées)
    // ==========================================
    let title = if invoice.is_proforma { "FACTURE PRO FORMA" } else { "FACTURATION" };
    layer.set_fill_color(blue_primary.clone());
    layer.use_text(title, 15.0, Mm(132.0), Mm(275.0), &font_bold);

    // Ligne 1 : En-tête bleu (N° FACTURE | DATE)
    draw_polygon(
        &layer,
        vec![
            point(120.0, 268.0),
            point(192.0, 268.0),
            point(192.0, 262.0),
            point(120.0, 262.0),
        ],
        PolygonMode::Fill,
    );
    layer.set_fill_color(white.clone());
    layer.use_text("N° DE FACTURE", 7.5, Mm(128.0), Mm(264.0), &font_bold);
    layer.use_text("DATE", 7.5, Mm(168.0), Mm(264.0), &font_bold);

    // Ligne 1 : Valeurs
    draw_polygon(
        &layer,
        vec![
            point(120.0, 262.0),
            point(192.0, 262.0),
            point(192.0, 256.0),
            point(120.0, 256.0),
        ],
        PolygonMode::Fill,
    );
    layer.set_fill_color(dark_text.clone());
    layer.use_text(&invoice.invoice_number, 8.0, Mm(132.0), Mm(258.0), &font_regular);
    layer.use_text(&invoice.date, 8.0, Mm(165.0), Mm(258.0), &font_regular);

    // Ligne 2 : En-tête bleu (DEVISE | MODALITÉS)
    layer.set_fill_color(blue_primary.clone());
    draw_polygon(
        &layer,
        vec![
            point(120.0, 256.0),
            point(192.0, 256.0),
            point(192.0, 250.0),
            point(120.0, 250.0),
        ],
        PolygonMode::Fill,
    );
    layer.set_fill_color(white.clone());
    layer.use_text("DEVISE", 7.5, Mm(132.0), Mm(252.0), &font_bold);
    layer.use_text("MODALITÉS", 7.5, Mm(162.0), Mm(252.0), &font_bold);

    // Ligne 2 : Valeurs
    layer.set_fill_color(dark_text.clone());
    layer.use_text(&settings.currency, 8.0, Mm(134.0), Mm(246.0), &font_regular);
    layer.use_text("Comptant", 8.0, Mm(164.0), Mm(246.0), &font_regular);

    // Cadre extérieur pour le tableau métadonnées
    draw_rect_outline(&layer, 120.0, 268.0, 192.0, 244.0, blue_primary.clone(), 0.3);

    // ==========================================
    // 3. BLOCS "FACTURE POUR" ET "EXPÉDIÉ À"
    // ==========================================
    let client_y = 236.0;

    // Bloc "FACTURE POUR :"
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
    let table_top = 212.0;
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

        // Écriture du produit si disponible
        if i < invoice.items.len() {
            let item = &invoice.items[i];
            layer.set_fill_color(dark_text.clone());
            layer.use_text(&item.product_name, 8.0, Mm(col_desc + 2.0), Mm(current_y - 4.0), &font_regular);
            layer.use_text(item.quantity.to_string(), 8.0, Mm(col_qte + 8.0), Mm(current_y - 4.0), &font_regular);
            layer.use_text(format_amount(item.unit_price), 8.0, Mm(col_pu + 10.0), Mm(current_y - 4.0), &font_regular);
            layer.use_text(format_amount(item.subtotal), 8.0, Mm(col_montant + 8.0), Mm(current_y - 4.0), &font_regular);
        } else {
            // Lignes vides avec '0,00' à droite comme dans le modèle Smartsheet
            layer.set_fill_color(Color::Rgb(Rgb::new(0.65, 0.65, 0.65, None)));
            layer.use_text("0,00", 7.5, Mm(col_montant + 12.0), Mm(current_y - 4.0), &font_regular);
        }

        // Ligne horizontale grise
        draw_line(&layer, col_desc, line_y, col_end, line_y, gray_border.clone(), 0.15);
        current_y = line_y;
    }

    // Lignes verticales du tableau
    for &x in &[col_desc, col_qte, col_pu, col_montant, col_end] {
        draw_line(&layer, x, table_top, x, table_bottom, gray_border.clone(), 0.15);
    }

    // ==========================================
    // 5. MERCI & BLOC TOTALS (SOUS-TOTAL, TAXE, TOTAL)
    // ==========================================
    // Message MERCI en bleu
    layer.set_fill_color(blue_primary.clone());
    layer.use_text("MERCI", 14.0, Mm(50.0), Mm(60.0), &font_bold);

    // Tableau Totaux (Bas Droite)
    let totals_top = 80.0;
    
    // Sous-total
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
    layer.set_fill_color(dark_text.clone());
    layer.use_text("SOUS-TOTAL", 8.0, Mm(124.0), Mm(totals_top - 4.2), &font_bold);
    layer.use_text(format_amount(invoice.total), 8.0, Mm(170.0), Mm(totals_top - 4.2), &font_regular);

    // Ligne Taxe
    draw_line(&layer, 120.0, totals_top - 6.0, 192.0, totals_top - 6.0, gray_border.clone(), 0.2);
    layer.use_text("REMISE / TAXE", 8.0, Mm(124.0), Mm(totals_top - 10.2), &font_bold);
    layer.use_text("0,00", 8.0, Mm(175.0), Mm(totals_top - 10.2), &font_regular);

    // Ligne TOTAL (Bleu Clair)
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

/// Logo Quincaillerie stylisé (Arc de cercle + Icône outil/brouette)
fn draw_tool_icon(layer: &PdfLayerReference, x: f32, y: f32, color: Color) {
    layer.set_fill_color(color.clone());
    layer.set_outline_color(color);
    layer.set_outline_thickness(2.5);

    // Forme de la brouette
    draw_polygon(
        layer,
        vec![
            point(x + 2.0, y),
            point(x + 14.0, y),
            point(x + 11.0, y - 5.0),
            point(x + 4.0, y - 5.0),
        ],
        PolygonMode::Fill,
    );

    // Roue de la brouette
    draw_polygon(
        layer,
        vec![
            point(x + 4.0, y - 4.0),
            point(x + 6.5, y - 7.0),
            point(x + 4.0, y - 9.0),
            point(x + 1.5, y - 7.0),
        ],
        PolygonMode::Stroke,
    );

    // Poignées de manutention
    let handle = Line {
        points: vec![point(x + 11.0, y - 2.0), point(x + 17.0, y + 4.0)],
        is_closed: false,
    };
    layer.add_line(handle);
}
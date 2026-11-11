//! Home tab: recommended tools, open card, pinned folders and recent files (local only, never
//! another app's list).

use egui::{Align2, CornerRadius, Rect, Sense, Stroke, vec2};
use pdfcraft_engine::catalog;

use crate::theme::{self, Tokens};
use crate::{PdfCraftApp, icons, panels::human_size, widgets};

const RECOMMENDED: [&str; 4] = ["organize", "comment", "form", "protect"];

/// A rounded rectangle stroked as short dashes.
fn dashed_rect(painter: &egui::Painter, rect: Rect, color: egui::Color32) {
    let dash = 7.0;
    let gap = 5.0;
    let inset = rect.shrink(1.0);
    let edges = [
        (inset.left_top(), inset.right_top()),
        (inset.right_top(), inset.right_bottom()),
        (inset.right_bottom(), inset.left_bottom()),
        (inset.left_bottom(), inset.left_top()),
    ];
    for (a, b) in edges {
        let delta = b - a;
        let len = delta.length();
        if len < 1.0 {
            continue;
        }
        let dir = delta / len;
        let mut t = 0.0;
        while t < len {
            let end = (t + dash).min(len);
            painter.line_segment([a + dir * t, a + dir * end], Stroke::new(1.25, color));
            t += dash + gap;
        }
    }
}

/// A soft shadow that rises on hover and settles when the card is pressed.
fn card_lift(ui: &egui::Ui, t: &Tokens, rect: Rect, press: theme::Press) {
    let lift = press.hover * (1.0 - press.down);
    if lift > 0.04 {
        let shadow = egui::Shadow { offset: [0, 3], blur: (14.0 * lift) as u8, spread: 0, color: t.page_shadow.gamma_multiply(0.85 * lift) };
        ui.painter().add(shadow.as_shape(rect, CornerRadius::same(10)));
    }
}

pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let avail = ui.available_width();
        let column = avail.min(960.0);
        let side = ((avail - column) * 0.5).max(28.0);
        ui.horizontal(|ui| {
            ui.add_space(side);
            ui.vertical(|ui| {
                ui.set_width(column);
                ui.add_space(36.0);
                ui.label(egui::RichText::new("Welcome to dpdf").font(theme::semibold(28.0)).color(t.text));
                ui.add_space(18.0);

                let drop_w = ui.available_width();
                let (drop_rect, drop_resp) = ui.allocate_exact_size(vec2(drop_w, 220.0), Sense::click());
                drop_resp.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Drop PDF files here to open, or click to browse"))
                });
                let drop_press = theme::Press::track(ui, &drop_resp);
                card_lift(ui, &t, drop_rect, drop_press);
                let drop_hovered = drop_resp.hovered() || ui.input(|i| !i.raw.hovered_files.is_empty());
                let drop_fill = if drop_hovered { t.selected } else { drop_press.fill(t.card, t.hover, t.pressed) };
                ui.painter().rect_filled(drop_rect, CornerRadius::same(12), drop_fill);
                let stroke_color = if drop_hovered { t.accent } else { t.border };
                if drop_hovered {
                    ui.painter().rect_stroke(drop_rect, CornerRadius::same(12), Stroke::new(1.5, stroke_color), egui::StrokeKind::Inside);
                } else {
                    dashed_rect(ui.painter(), drop_rect, stroke_color);
                }
                let drop_body = drop_rect.translate(drop_press.offset());
                icons::paint(
                    ui,
                    Rect::from_center_size(drop_body.center() - vec2(0.0, 18.0), vec2(36.0, 36.0)),
                    "file-text",
                    32.0,
                    if drop_hovered { t.accent } else { t.icon },
                );
                ui.painter().text(
                    drop_body.center() + vec2(0.0, 22.0),
                    Align2::CENTER_CENTER,
                    tl!("Drop a PDF or click to open"),
                    theme::semibold(15.0),
                    if drop_hovered { t.accent_text } else { t.text },
                );
                if theme::hand(drop_resp).clicked() {
                    app.open_dialog();
                }
                ui.add_space(28.0);

                let mut clear = false;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tl!("Recent")).font(theme::semibold(16.0)).color(t.text));
                    if !app.recent.is_empty() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let resp = widgets::ghost_button(ui, "trash-2", tl!("Clear"));
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Clear Recent Files")));
                            clear = resp.on_hover_text(tl!("Clear Recent Files")).clicked();
                        });
                    }
                });
                ui.add_space(10.0);
                if app.recent.is_empty() {
                    ui.label(egui::RichText::new(tl!("Files you open in dpdf appear here. Drop a PDF anywhere to open it.")).color(t.text_muted));
                }
                let mut open = None;
                let mut remove = None;
                let card_w = ((ui.available_width() - 36.0) / 4.0).clamp(200.0, 280.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                    for r in &app.recent {
                        let (rect, resp) = ui.allocate_exact_size(vec2(card_w, 72.0), Sense::click());
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &r.name));
                        let press = theme::Press::track(ui, &resp);
                        let hovered = ui.rect_contains_pointer(rect);
                        let fill = if hovered { press.fill(t.card, t.hover, t.pressed) } else { t.card };
                        ui.painter().rect(rect, CornerRadius::same(10), fill, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
                        if hovered {
                            let x_rect = Rect::from_center_size(rect.right_center() - vec2(22.0, 0.0), vec2(28.0, 28.0));
                            let x = ui.interact(x_rect, ui.id().with(("remove-recent", &r.path)), Sense::click());
                            let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &r.name)]);
                            x.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
                            if x.hovered() {
                                ui.painter().rect_filled(x_rect, CornerRadius::same(6), t.pressed);
                            }
                            icons::paint(ui, x_rect.shrink(6.0), "x", 16.0, t.icon);
                            if x.on_hover_text(tl!("Remove from Recent")).clicked() {
                                remove = Some(r.path.clone());
                            }
                        }
                        let body = rect.translate(press.offset());
                        let thumb = Rect::from_min_size(body.min + vec2(10.0, 12.0), vec2(36.0, 48.0));
                        ui.painter().rect(thumb, CornerRadius::same(4), t.pasteboard, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
                        icons::paint(ui, Rect::from_center_size(thumb.center(), vec2(18.0, 18.0)), "file-text", 16.0, t.accent);
                        let text_left = body.left() + 56.0;
                        let text_right = body.right() - if hovered { 40.0 } else { 12.0 };
                        let width = (text_right - text_left).max(1.0);
                        widgets::row_text(
                            ui,
                            egui::pos2(text_left, body.center().y - 10.0),
                            crate::bidi::visual(&r.name),
                            theme::medium(13.0),
                            t.text,
                            width,
                        );
                        let detail = format!(
                            "{}  ·  {}",
                            crate::i18n::trn(crate::i18n::current(), r.pages as u64, "{n} page", "{n} pages"),
                            human_size(r.size)
                        );
                        widgets::row_text(ui, egui::pos2(text_left, body.center().y + 10.0), detail, theme::regular(11.5), t.text_muted, width);
                        if theme::hand(resp).on_hover_text(&r.path).clicked() {
                            open = Some(r.path.clone());
                        }
                    }
                });
                if let Some(p) = open {
                    app.open_recent(&p);
                }
                if let Some(p) = remove {
                    app.recent.retain(|r| r.path != p);
                }
                if clear {
                    app.execute("file.clear_recent");
                }
                ui.add_space(28.0);

                egui::Frame::NONE
                    .fill(t.card)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(egui::Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(egui::RichText::new(tl!("Recommended tools")).font(theme::semibold(15.0)));
                        ui.add_space(10.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(14.0, 14.0);
                            for id in RECOMMENDED {
                                let Some(g) = catalog::group(id) else { continue };
                                let (rect, resp) = ui.allocate_exact_size(vec2(190.0, 104.0), Sense::click());
                                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!(g.label)));
                                let press = theme::Press::track(ui, &resp);
                                card_lift(ui, &t, rect, press);
                                let fill = press.fill(t.card, t.hover, t.pressed);
                                ui.painter().rect(rect, CornerRadius::same(10), fill, Stroke::new(1.0, t.divider), egui::StrokeKind::Inside);
                                let body = rect.translate(press.offset());
                                let color = egui::Color32::from_rgb(g.hue[0], g.hue[1], g.hue[2]);
                                icons::paint(ui, Rect::from_min_size(body.min + vec2(14.0, 14.0), vec2(22.0, 22.0)), g.icon, 21.0, color);
                                let mut title_job = egui::text::LayoutJob::simple_singleline(tl!(g.label).to_owned(), theme::semibold(13.5), t.text);
                                title_job.wrap = egui::text::TextWrapping {
                                    max_width: body.width() - 58.0,
                                    max_rows: 1,
                                    break_anywhere: true,
                                    ..Default::default()
                                };
                                let title = ui.fonts_mut(|f| f.layout_job(title_job));
                                let title_elided = title.elided;
                                ui.painter().galley(body.min + vec2(44.0, 25.0 - title.size().y * 0.5), title, t.text);
                                let blurb = g
                                    .sections
                                    .first()
                                    .map(|s| s.items.iter().take(3).map(|i| tl!(i.label)).collect::<Vec<_>>().join(" · "))
                                    .unwrap_or_default();
                                // Fixed-height cards must leave room for their action in every language.
                                let mut job = egui::text::LayoutJob::simple(blurb.clone(), theme::regular(11.5), t.text_muted, body.width() - 28.0);
                                job.wrap.max_rows = 2;
                                let galley = ui.fonts_mut(|f| f.layout_job(job));
                                let blurb_elided = galley.elided;
                                ui.painter().galley(body.min + vec2(14.0, 46.0), galley, t.text_muted);
                                ui.painter().text(
                                    body.left_bottom() + vec2(14.0, -14.0),
                                    Align2::LEFT_CENTER,
                                    tl!("Use now"),
                                    theme::medium(12.0),
                                    t.accent_text,
                                );
                                let resp = if title_elided || blurb_elided { resp.on_hover_text(format!("{}\n{blurb}", tl!(g.label))) } else { resp };
                                if theme::hand(resp).clicked() {
                                    app.select_tool(g.id);
                                }
                            }
                            let (rect, resp) = ui.allocate_exact_size(vec2(170.0, 104.0), Sense::click());
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Open file")));
                            let press = theme::Press::track(ui, &resp);
                            card_lift(ui, &t, rect, press);
                            ui.painter().rect(
                                rect,
                                CornerRadius::same(10),
                                press.fill(t.pasteboard, t.hover, t.pressed),
                                Stroke::new(1.0, t.divider),
                                egui::StrokeKind::Inside,
                            );
                            let body = rect.translate(press.offset());
                            icons::paint(ui, Rect::from_center_size(body.center() - vec2(0.0, 16.0), vec2(28.0, 28.0)), "folder-open", 26.0, t.icon);
                            ui.painter().text(
                                body.center() + vec2(0.0, 22.0),
                                Align2::CENTER_CENTER,
                                tl!("Open file"),
                                theme::semibold(13.0),
                                t.text,
                            );
                            if theme::hand(resp).clicked() {
                                app.open_dialog();
                            }
                        });
                    });

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.add_space(28.0);
                    crate::folders_ui::section(app, ui);
                }
                ui.add_space(24.0);
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translated_card_description_leaves_room_for_the_action() {
        let de = crate::i18n::Lang::from_code("de").unwrap();
        crate::i18n::set_current(de);
        let ctx = egui::Context::default();
        ctx.set_fonts(theme::font_definitions());
        let mut app = PdfCraftApp::new();
        let blurb = catalog::group("form")
            .unwrap()
            .sections
            .first()
            .unwrap()
            .items
            .iter()
            .take(3)
            .map(|item| crate::i18n::tr(de, item.label))
            .collect::<Vec<_>>()
            .join(" · ");
        let output = ctx
            .run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(1440.0, 900.0))), ..Default::default() }, |ui| {
                show(&mut app, ui)
            });
        crate::i18n::set_current(crate::i18n::Lang::EN);
        let text_shapes: Vec<_> =
            output.shapes.iter().filter_map(|shape| if let egui::Shape::Text(text) = &shape.shape { Some(text) } else { None }).collect();
        let description = text_shapes.iter().find(|text| text.galley.job.text == blurb).unwrap();
        let action = text_shapes
            .iter()
            .find(|text| text.galley.job.text == crate::i18n::tr(de, "Use now") && (text.pos.x - description.pos.x).abs() < 0.1)
            .unwrap();
        assert!(description.galley.rows.len() <= 2);
        assert!(
            description.pos.y + description.galley.size().y + 4.0 <= action.pos.y,
            "the translated description must leave a visible gap before its action"
        );
        output.drop_without_applying_deltas();
    }
}

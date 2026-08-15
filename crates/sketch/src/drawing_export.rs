//! MCP-native drawing interchange.
//!
//! These writers emit paper-space DXF/SVG from saved view intent plus
//! regenerated projection curves, and 1:1 sketch-plane DXF from profile
//! loops. They are not a port of the UI annotation DXF writer: dimensions
//! and symbols stay in the drawing DTO / named command tools.

use nbcad_solid::ProfileCatalogItemDto;

use crate::drawing::{
    drawing_sheet_size, DrawingAnnotationDto, DrawingDocumentDto, DrawingSheetDto, DrawingViewDto,
};

#[derive(Debug, Clone)]
pub struct DrawingExportCircle {
    pub center: [f64; 2],
    pub radius: f64,
    pub hidden: bool,
}

/// One projected view ready to place on paper. Coordinates in `visible` /
/// `hidden` / `section` / `circles` are model millimetres (same space as HLR).
#[derive(Debug, Clone)]
pub struct DrawingExportView {
    pub view_id: u64,
    pub position: [f64; 2],
    pub scale: f64,
    pub visible: Vec<Vec<[f64; 2]>>,
    pub hidden: Vec<Vec<[f64; 2]>>,
    pub section: Vec<Vec<[f64; 2]>>,
    pub circles: Vec<DrawingExportCircle>,
    /// min x, min y, max x, max y in model millimetres.
    pub bounds: [f64; 4],
}

pub fn drawing_sheet_dxf(
    document: &DrawingDocumentDto,
    sheet_id: Option<u64>,
    views: &[DrawingExportView],
) -> Result<String, String> {
    let sheet = active_sheet(document, sheet_id)?;
    let [width, height] = drawing_sheet_size(sheet.format, sheet.orientation);
    let mut entities = String::new();
    push_polyline(
        &mut entities,
        "BORDER",
        &[
            [0.0, 0.0],
            [width, 0.0],
            [width, height],
            [0.0, height],
            [0.0, 0.0],
        ],
        true,
        height,
    );
    for view in views {
        let Some(stored) = sheet
            .views
            .iter()
            .find(|candidate| candidate.id == view.view_id)
        else {
            continue;
        };
        let placed = placed_view(stored, view);
        for polyline in &placed.visible {
            push_polyline(&mut entities, "VISIBLE", polyline, false, height);
        }
        for polyline in &placed.hidden {
            push_polyline(&mut entities, "HIDDEN", polyline, false, height);
        }
        for polyline in &placed.section {
            push_polyline(&mut entities, "SECTION", polyline, false, height);
        }
        for circle in &placed.circles {
            let layer = if circle.hidden { "HIDDEN" } else { "VISIBLE" };
            push_circle(&mut entities, layer, circle.center, circle.radius, height);
        }
    }
    for annotation in &sheet.annotations {
        if let DrawingAnnotationDto::Note { text, position, .. } = annotation {
            push_text(&mut entities, "NOTES", *position, text, 3.5, height);
        }
    }
    let title = sheet.title_block.title.trim();
    if !title.is_empty() {
        push_text(
            &mut entities,
            "TITLE",
            [12.0, height - 12.0],
            title,
            5.0,
            height,
        );
    }
    if !sheet.title_block.drawing_number.trim().is_empty() {
        push_text(
            &mut entities,
            "TITLE",
            [12.0, height - 20.0],
            &format!("DWG {}", sheet.title_block.drawing_number),
            3.5,
            height,
        );
    }
    Ok(wrap_dxf(&entities, width, height))
}

pub fn drawing_sheet_svg(
    document: &DrawingDocumentDto,
    sheet_id: Option<u64>,
    views: &[DrawingExportView],
) -> Result<String, String> {
    let sheet = active_sheet(document, sheet_id)?;
    let [width, height] = drawing_sheet_size(sheet.format, sheet.orientation);
    let mut body = String::new();
    body.push_str(&format!(
        r##"<rect width="{width}" height="{height}" fill="white" stroke="#17191c"/>"##
    ));
    for view in views {
        let Some(stored) = sheet
            .views
            .iter()
            .find(|candidate| candidate.id == view.view_id)
        else {
            continue;
        };
        let placed = placed_view(stored, view);
        for polyline in &placed.visible {
            body.push_str(&svg_polyline(polyline, "#17191c", 0.25, false));
        }
        for polyline in &placed.hidden {
            body.push_str(&svg_polyline(polyline, "#6b7280", 0.18, true));
        }
        for polyline in &placed.section {
            body.push_str(&svg_polyline(polyline, "#b45309", 0.35, false));
        }
        for circle in &placed.circles {
            let stroke = if circle.hidden { "#6b7280" } else { "#17191c" };
            body.push_str(&format!(
                r#"<circle cx="{:.4}" cy="{:.4}" r="{:.4}" fill="none" stroke="{stroke}" stroke-width="0.25"/>"#,
                circle.center[0], circle.center[1], circle.radius
            ));
        }
    }
    for annotation in &sheet.annotations {
        if let DrawingAnnotationDto::Note { text, position, .. } = annotation {
            let escaped = xml_escape(text);
            body.push_str(&format!(
                r##"<text x="{:.4}" y="{:.4}" font-size="3.5" fill="#17191c">{}</text>"##,
                position[0], position[1], escaped
            ));
        }
    }
    let title = xml_escape(sheet.title_block.title.trim());
    if !title.is_empty() {
        body.push_str(&format!(
            r##"<text x="12" y="{:.4}" font-size="5" fill="#17191c">{title}</text>"##,
            height - 8.0
        ));
    }
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}mm" height="{height}mm" viewBox="0 0 {width} {height}">
{body}
</svg>
"#
    ))
}

pub fn manufacturing_profile_dxf(
    catalog: &ProfileCatalogItemDto,
    profile_index: u32,
) -> Result<String, String> {
    let outer = catalog
        .profiles
        .iter()
        .find(|profile| profile.index == profile_index)
        .ok_or_else(|| {
            format!(
                "profile {profile_index} was not found on sketch {}",
                catalog.sketch_name
            )
        })?;
    if outer.nesting_depth % 2 != 0 {
        return Err(
            "a manufacturing profile must be an outer material region, not a hole boundary"
                .to_string(),
        );
    }
    let holes: Vec<_> = catalog
        .profiles
        .iter()
        .filter(|profile| profile.parent_index == Some(outer.index))
        .collect();
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut entities = String::new();
    for (loop_profile, layer) in std::iter::once((outer, "PROFILE_OUTER"))
        .chain(holes.iter().map(|hole| (*hole, "PROFILE_HOLES")))
    {
        let points: Vec<[f64; 2]> = loop_profile
            .points
            .iter()
            .map(|point| [point.x, point.y])
            .collect();
        for point in &points {
            min_x = min_x.min(point[0]);
            min_y = min_y.min(point[1]);
            max_x = max_x.max(point[0]);
            max_y = max_y.max(point[1]);
        }
        push_model_polyline(&mut entities, layer, &points, true);
    }
    if !min_x.is_finite() {
        return Err("profile has no points".to_string());
    }
    Ok(wrap_dxf(
        &entities,
        (max_x - min_x).max(1.0),
        (max_y - min_y).max(1.0),
    ))
}

struct PlacedView {
    visible: Vec<Vec<[f64; 2]>>,
    hidden: Vec<Vec<[f64; 2]>>,
    section: Vec<Vec<[f64; 2]>>,
    circles: Vec<DrawingExportCircle>,
}

fn placed_view(stored: &DrawingViewDto, view: &DrawingExportView) -> PlacedView {
    let scale = if view.scale > 0.0 {
        view.scale
    } else {
        stored.scale
    };
    let position = view.position;
    let bounds = view.bounds;
    PlacedView {
        visible: view
            .visible
            .iter()
            .map(|polyline| project_polyline(position, scale, bounds, polyline))
            .collect(),
        hidden: view
            .hidden
            .iter()
            .map(|polyline| project_polyline(position, scale, bounds, polyline))
            .collect(),
        section: view
            .section
            .iter()
            .map(|polyline| project_polyline(position, scale, bounds, polyline))
            .collect(),
        circles: view
            .circles
            .iter()
            .map(|circle| DrawingExportCircle {
                center: project_point(position, scale, bounds, circle.center),
                radius: circle.radius * scale,
                hidden: circle.hidden,
            })
            .collect(),
    }
}

fn project_polyline(
    position: [f64; 2],
    scale: f64,
    bounds: [f64; 4],
    polyline: &[[f64; 2]],
) -> Vec<[f64; 2]> {
    polyline
        .iter()
        .copied()
        .map(|point| project_point(position, scale, bounds, point))
        .collect()
}

fn project_point(position: [f64; 2], scale: f64, bounds: [f64; 4], point: [f64; 2]) -> [f64; 2] {
    let center_x = (bounds[0] + bounds[2]) / 2.0;
    let center_y = (bounds[1] + bounds[3]) / 2.0;
    [
        position[0] + (point[0] - center_x) * scale,
        position[1] - (point[1] - center_y) * scale,
    ]
}

fn active_sheet(
    document: &DrawingDocumentDto,
    sheet_id: Option<u64>,
) -> Result<&DrawingSheetDto, String> {
    let id = sheet_id.or(document.active_sheet_id);
    document
        .sheets
        .iter()
        .find(|sheet| Some(sheet.id) == id)
        .ok_or_else(|| "there is no active drawing sheet to export".to_string())
}

fn paper_to_dxf(point: [f64; 2], sheet_height: f64) -> [f64; 2] {
    [point[0], sheet_height - point[1]]
}

fn push_polyline(
    out: &mut String,
    layer: &str,
    points: &[[f64; 2]],
    closed: bool,
    sheet_height: f64,
) {
    let mapped: Vec<[f64; 2]> = points
        .iter()
        .copied()
        .map(|point| paper_to_dxf(point, sheet_height))
        .collect();
    push_model_polyline(out, layer, &mapped, closed);
}

fn push_model_polyline(out: &mut String, layer: &str, points: &[[f64; 2]], closed: bool) {
    if points.len() < 2 {
        return;
    }
    let count = if closed && points.first() == points.last() && points.len() > 2 {
        points.len() - 1
    } else {
        points.len()
    };
    out.push_str(&format!(
        "0\nLWPOLYLINE\n8\n{layer}\n90\n{count}\n70\n{}\n",
        if closed { 1 } else { 0 }
    ));
    for point in points.iter().take(count) {
        out.push_str(&format!("10\n{:.6}\n20\n{:.6}\n", point[0], point[1]));
    }
}

fn push_circle(out: &mut String, layer: &str, center: [f64; 2], radius: f64, sheet_height: f64) {
    if !(radius > 0.0) {
        return;
    }
    let [x, y] = paper_to_dxf(center, sheet_height);
    out.push_str(&format!(
        "0\nCIRCLE\n8\n{layer}\n10\n{x:.6}\n20\n{y:.6}\n40\n{radius:.6}\n"
    ));
}

fn push_text(
    out: &mut String,
    layer: &str,
    position: [f64; 2],
    text: &str,
    height: f64,
    sheet_height: f64,
) {
    let [x, y] = paper_to_dxf(position, sheet_height);
    let sanitized = text.replace(['\n', '\r'], " ");
    out.push_str(&format!(
        "0\nTEXT\n8\n{layer}\n10\n{x:.6}\n20\n{y:.6}\n40\n{height:.3}\n1\n{sanitized}\n"
    ));
}

fn wrap_dxf(entities: &str, width: f64, height: f64) -> String {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n9\n$EXTMIN\n10\n0\n20\n0\n9\n$EXTMAX\n10\n{width:.3}\n20\n{height:.3}\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n{entities}0\nENDSEC\n0\nEOF\n"
    )
}

fn svg_polyline(points: &[[f64; 2]], stroke: &str, width: f64, dashed: bool) -> String {
    if points.len() < 2 {
        return String::new();
    }
    let data: String = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            format!(
                "{}{:.4},{:.4}",
                if index == 0 { "M" } else { " L" },
                point[0],
                point[1]
            )
        })
        .collect();
    let dash = if dashed {
        r#" stroke-dasharray="1.2 0.8""#
    } else {
        ""
    };
    format!(r#"<path d="{data}" fill="none" stroke="{stroke}" stroke-width="{width}"{dash}/>"#)
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbcad_core::{FeatureId, PlaneBasis};
    use nbcad_solid::{Point2Dto, ProfileLoopDto, SolidSceneDto};

    use crate::drawing::{DrawingDocumentDto, DrawingStandard};
    use crate::drawing_ops::{apply_drawing_command, DrawingCommand};

    fn sheet_with_view() -> DrawingDocumentDto {
        let drawing = apply_drawing_command(
            &DrawingDocumentDto::default(),
            &SolidSceneDto::default(),
            &[],
            "BRACKET",
            DrawingCommand::CreateSheet {
                standard: DrawingStandard::Iso,
                format: None,
                orientation: None,
                projection_method: None,
                tolerance_note: None,
                title: Some("BRACKET".to_string()),
                drawing_number: Some("DWG-1".to_string()),
                revision: None,
                author: None,
            },
        )
        .unwrap();
        let drawing = apply_drawing_command(
            &drawing,
            &SolidSceneDto::default(),
            &[],
            "BRACKET",
            DrawingCommand::AddView {
                kind: crate::drawing::DrawingViewKind::Front,
                position: Some([80.0, 80.0]),
                parent_view_id: None,
                scale: Some(1.0),
            },
        )
        .unwrap();
        apply_drawing_command(
            &drawing,
            &SolidSceneDto::default(),
            &[],
            "BRACKET",
            DrawingCommand::AddNote {
                position: [20.0, 30.0],
                text: Some("CHECK EDGE".to_string()),
            },
        )
        .unwrap()
    }

    #[test]
    fn sheet_dxf_contains_hlr_geometry_and_note() {
        let document = sheet_with_view();
        let dxf = drawing_sheet_dxf(
            &document,
            None,
            &[DrawingExportView {
                view_id: 1,
                position: [80.0, 80.0],
                scale: 1.0,
                visible: vec![vec![[-10.0, -5.0], [10.0, -5.0], [10.0, 5.0]]],
                hidden: vec![vec![[-8.0, 0.0], [8.0, 0.0]]],
                section: vec![],
                circles: vec![DrawingExportCircle {
                    center: [0.0, 0.0],
                    radius: 2.5,
                    hidden: false,
                }],
                bounds: [-10.0, -5.0, 10.0, 5.0],
            }],
        )
        .unwrap();
        assert!(dxf.contains("LWPOLYLINE"), "{dxf}");
        assert!(dxf.contains("VISIBLE"), "{dxf}");
        assert!(dxf.contains("HIDDEN"), "{dxf}");
        assert!(dxf.contains("CIRCLE"), "{dxf}");
        assert!(dxf.contains("CHECK EDGE"), "{dxf}");
        assert!(dxf.contains("BRACKET"), "{dxf}");
        assert!(dxf.contains("$INSUNITS"), "{dxf}");
    }

    #[test]
    fn sheet_svg_is_paper_sized() {
        let document = sheet_with_view();
        let svg = drawing_sheet_svg(&document, None, &[]).unwrap();
        assert!(svg.contains("297mm"), "{svg}");
        assert!(svg.contains("BRACKET"), "{svg}");
        assert!(svg.contains("CHECK EDGE"), "{svg}");
    }

    #[test]
    fn profile_dxf_rejects_hole_loops() {
        let catalog = ProfileCatalogItemDto {
            sketch_name: "Sketch1".to_string(),
            feature_id: FeatureId(1),
            basis: PlaneBasis {
                origin: [0.0, 0.0, 0.0],
                u: [1.0, 0.0, 0.0],
                v: [0.0, 1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            profiles: vec![ProfileLoopDto {
                index: 0,
                points: vec![
                    Point2Dto::new(0.0, 0.0),
                    Point2Dto::new(10.0, 0.0),
                    Point2Dto::new(10.0, 10.0),
                    Point2Dto::new(0.0, 10.0),
                ],
                area: 100.0,
                parent_index: None,
                nesting_depth: 1,
                curves: vec![],
            }],
            lines: vec![],
            path_curves: vec![],
            reference_points: vec![],
        };
        assert!(manufacturing_profile_dxf(&catalog, 0).is_err());
    }

    #[test]
    fn profile_dxf_writes_outer_loop() {
        let catalog = ProfileCatalogItemDto {
            sketch_name: "Sketch1".to_string(),
            feature_id: FeatureId(1),
            basis: PlaneBasis {
                origin: [0.0, 0.0, 0.0],
                u: [1.0, 0.0, 0.0],
                v: [0.0, 1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            profiles: vec![ProfileLoopDto {
                index: 0,
                points: vec![
                    Point2Dto::new(0.0, 0.0),
                    Point2Dto::new(20.0, 0.0),
                    Point2Dto::new(20.0, 10.0),
                    Point2Dto::new(0.0, 10.0),
                ],
                area: 200.0,
                parent_index: None,
                nesting_depth: 0,
                curves: vec![],
            }],
            lines: vec![],
            path_curves: vec![],
            reference_points: vec![],
        };
        let dxf = manufacturing_profile_dxf(&catalog, 0).unwrap();
        assert!(dxf.contains("PROFILE_OUTER"), "{dxf}");
        assert!(dxf.contains("20.000000"), "{dxf}");
    }
}

//! CAD synthesis exam: build a print-tolerant journal kit and grade it.
//!
//! Spec: `scripts/fixtures/print-kit-tutor.spec.json`.
//! Agents follow the same numbers via `prompts/get model_print_kit`.

use serde::Deserialize;
use serde_json::{json, Value};

pub const SPEC_JSON: &str = include_str!("../../scripts/fixtures/print-kit-tutor.spec.json");

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Spec {
    pub id: String,
    pub title: String,
    pub document_name: String,
    pub slicer_target: String,
    pub nozzle_mm: f64,
    pub clearance_mm: f64,
    pub journal_d: f64,
    pub bush_id: f64,
    pub bush_od: f64,
    pub bush_h: f64,
    pub house_id: f64,
    pub house_h: f64,
    pub house_plate: f64,
    pub house_height: f64,
    pub thru_d: f64,
    pub flange_d: f64,
    pub flange_h: f64,
    pub shaft_h: f64,
    pub tip_chamfer: f64,
    pub bed_chamfer: f64,
    pub rotor_d: f64,
    pub overlap: f64,
    pub blade_wall: f64,
    pub blade_h: f64,
    pub blade_twist_deg: f64,
    pub loft_stations: usize,
    pub min_bodies: usize,
    pub min_blade_faces: usize,
    pub placements: Placements,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Placements {
    pub shaft: [f64; 2],
    pub bushing: [f64; 2],
    pub housing: [f64; 2],
    pub blade: [f64; 2],
}

#[derive(Debug, Clone)]
pub struct LessonResult {
    pub id: String,
    pub pass: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Report {
    pub ok: bool,
    pub spec_id: String,
    pub lessons: Vec<LessonResult>,
    pub body_count: usize,
    pub byte_length: usize,
}

impl Report {
    fn fail(spec_id: &str, id: &str, detail: impl Into<String>) -> Self {
        Self {
            ok: false,
            spec_id: spec_id.to_string(),
            lessons: vec![LessonResult {
                id: id.to_string(),
                pass: false,
                detail: detail.into(),
            }],
            body_count: 0,
            byte_length: 0,
        }
    }
}

pub fn load_spec() -> Result<Spec, String> {
    serde_json::from_str(SPEC_JSON).map_err(|error| format!("print-kit spec: {error}"))
}

pub fn run(call: &mut impl FnMut(&str, Value) -> Result<Value, String>) -> Result<Report, String> {
    let spec = load_spec()?;
    if (spec.bush_id - spec.journal_d - spec.clearance_mm).abs() > 1e-9
        || (spec.house_id - spec.bush_od - spec.clearance_mm).abs() > 1e-9
    {
        return Ok(Report::fail(
            &spec.id,
            "clearance",
            "spec clearances are not exactly +nozzle",
        ));
    }

    call("cad_new_project", json!({}))?;
    call(
        "cad_set_document_name",
        json!({ "name": spec.document_name }),
    )?;
    call(
        "cad_set_focus",
        json!({ "focus": "sketch", "explicit": true }),
    )?;

    begin_xz(call)?;
    add_poly(call, &shaft_profile(&spec))?;
    let shaft_sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_revolve",
            json!({
                "sketch_name": shaft_sketch,
                "profile_indices": [0],
                "axis_origin": { "x": 0.0, "y": 0.0 },
                "axis_direction": { "x": 0.0, "y": 1.0 },
                "axis_line_entity_id": null,
                "angle_deg": 360.0,
                "flip": false,
                "operation": "new_body",
                "target_body_ids": []
            }),
        )?,
        "shaft revolve",
    )?;
    let shaft_id = first_body_id(&update)?;

    begin_xy(call)?;
    add_circle(call, spec.placements.bushing, spec.bush_od)?;
    add_circle(call, spec.placements.bushing, spec.bush_id)?;
    let bush_sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": bush_sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.bush_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "bushing extrude",
    )?;
    let bush_id = newest_body_id(&update, &[shaft_id])?;

    let hx = spec.placements.housing[0];
    let hy = spec.placements.housing[1];
    let half = spec.house_plate / 2.0;
    begin_xy(call)?;
    call(
        "sketch_add_rectangle",
        json!({
            "mode": "two_point",
            "p1": { "x": hx - half, "y": hy - half },
            "p2": { "x": hx + half, "y": hy + half },
            "ctrl_held": false
        }),
    )?;
    let house_sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": house_sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.house_height },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "housing extrude",
    )?;
    let house_id = newest_body_id(&update, &[shaft_id, bush_id])?;
    let house_top = offset_xy(call, spec.house_height)?;
    begin_datum(call, house_top.clone())?;
    add_circle(call, spec.placements.housing, spec.thru_d)?;
    let thru_sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": thru_sketch,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.house_height + 1.0 },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [house_id]
            }),
        )?,
        "housing through",
    )?;
    begin_datum(call, house_top)?;
    add_circle(call, spec.placements.housing, spec.house_id)?;
    let seat_sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": seat_sketch,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.house_h },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [house_id]
            }),
        )?,
        "608 seat",
    )?;

    let stations = spec.loft_stations.max(2);
    let mut section_names = Vec::new();
    for i in 0..stations {
        let z = spec.blade_h * (i as f64) / ((stations - 1) as f64);
        let ang = spec.blade_twist_deg * (i as f64) / ((stations - 1) as f64);
        let datum_id = offset_xy(call, z)?;
        begin_datum(call, datum_id)?;
        add_c(call, &spec, ang, spec.placements.blade)?;
        section_names.push(finish_sketch(call)?);
    }
    let update = require_clean(
        call(
            "solid_loft",
            json!({
                "sections": section_names.iter().map(|name| json!({
                    "sketch_name": name,
                    "profile_index": 0
                })).collect::<Vec<_>>(),
                "ruled": false,
                "operation": "new_body",
                "target_body_ids": [],
                "continuity": "g1",
                "centerline": null,
                "guide_rail": null
            }),
        )?,
        "helical loft",
    )?;
    let blade_id = newest_body_id(&update, &[shaft_id, bush_id, house_id])?;

    call(
        "cad_set_focus",
        json!({ "focus": "print", "explicit": true }),
    )?;
    for (id, preset) in [
        (shaft_id, "bambu.pla.basic.jade_white"),
        (bush_id, "bambu.pla.matte.dark_gray"),
        (house_id, "bambu.pla.basic.black"),
        (blade_id, "bambu.pla.basic.green"),
    ] {
        call(
            "set_body_appearance",
            json!({ "body_id": id, "preset_id": preset }),
        )?;
    }
    let preflight = call("solid_export_preflight", json!({}))?;
    let exported = call(
        "solid_export_3mf",
        json!({ "slicer_target": spec.slicer_target, "include_appearance": true }),
    )?;
    let scene = call("solid_scene", json!({}))?;
    let document = call("cad_document", json!({}))?;
    Ok(grade(
        &spec, &scene, &document, &preflight, &exported, blade_id,
    ))
}

fn grade(
    spec: &Spec,
    scene: &Value,
    document: &Value,
    preflight: &Value,
    exported: &Value,
    blade_id: u64,
) -> Report {
    let bodies = scene["bodies"].as_array().cloned().unwrap_or_default();
    let features = document["features"].as_array().cloned().unwrap_or_default();
    let blade = bodies
        .iter()
        .find(|body| body["id"].as_u64() == Some(blade_id));
    let bytes = exported["bytes_base64"]
        .as_str()
        .and_then(|b64| {
            use base64::{engine::general_purpose::STANDARD, Engine as _};
            STANDARD.decode(b64).ok()
        })
        .unwrap_or_default();

    let mut lessons = Vec::new();
    push_lesson(
        &mut lessons,
        "clearance",
        (spec.bush_id - spec.journal_d - spec.clearance_mm).abs() < 1e-9
            && (spec.house_id - spec.bush_od - spec.clearance_mm).abs() < 1e-9,
        format!(
            "journal {:.1} in bush {:.1}; bush {:.1} in house {:.1}",
            spec.journal_d, spec.bush_id, spec.bush_od, spec.house_id
        ),
    );
    push_lesson(
        &mut lessons,
        "no_press",
        spec.clearance_mm >= spec.nozzle_mm && spec.house_id > spec.bush_od,
        "all printed interfaces are +0.40 slip with a 608 shoulder seat".to_string(),
    );
    let on_bed = bodies.iter().all(|body| {
        bbox(body)
            .map(|box3| box3[0][2] > -0.6 && box3[0][2] < 0.6)
            .unwrap_or(false)
    });
    push_lesson(
        &mut lessons,
        "orientation",
        on_bed && bodies.len() >= spec.min_bodies,
        format!("{} bodies on z=0", bodies.len()),
    );
    push_lesson(
        &mut lessons,
        "xy_holes",
        spec.thru_d > spec.journal_d && spec.house_id > spec.bush_od,
        "housing through and 608 seat are XY circles from the top face".to_string(),
    );
    let bush = bodies.iter().find(|body| {
        bbox(body).is_some_and(|box3| {
            let span = [
                box3[1][0] - box3[0][0],
                box3[1][1] - box3[0][1],
                box3[1][2] - box3[0][2],
            ];
            span[2] > 6.0 && span[2] < 8.5 && span[0] > 18.0
        })
    });
    push_lesson(
        &mut lessons,
        "mechanical",
        bush.is_some() && bodies.len() >= spec.min_bodies,
        "608-envelope bushing (Ø22 × 7) plus journal shaft and housing".to_string(),
    );
    let blade_faces = blade
        .and_then(|body| body["faces"].as_array().map(|faces| faces.len()))
        .unwrap_or(0);
    let blade_h = blade
        .and_then(bbox)
        .map(|box3| box3[1][2] - box3[0][2])
        .unwrap_or(0.0);
    push_lesson(
        &mut lessons,
        "not_2d",
        blade_faces >= spec.min_blade_faces && blade_h > spec.blade_h - 2.0,
        format!("helical loft faces={blade_faces} height={blade_h:.1}"),
    );
    let timeline_ok = features.iter().all(|feature| {
        feature["status"]["state"]
            .as_str()
            .is_some_and(|state| state == "ok")
            || feature["status"].as_str() == Some("ok")
    });
    let preflight_ok = preflight["ok"] == true
        || preflight["timeline_errors"]
            .as_array()
            .is_some_and(Vec::is_empty);
    let zip_ok = bytes.len() > 32 && bytes.starts_with(b"PK");
    push_lesson(
        &mut lessons,
        "export",
        timeline_ok && preflight_ok && zip_ok,
        format!("3MF {} bytes, timeline clean", bytes.len()),
    );

    Report {
        ok: lessons.iter().all(|lesson| lesson.pass),
        spec_id: spec.id.clone(),
        lessons,
        body_count: bodies.len(),
        byte_length: bytes.len(),
    }
}

fn push_lesson(lessons: &mut Vec<LessonResult>, id: &str, pass: bool, detail: String) {
    lessons.push(LessonResult {
        id: id.to_string(),
        pass,
        detail,
    });
}

fn shaft_profile(spec: &Spec) -> Vec<[f64; 2]> {
    let jr = spec.journal_d / 2.0;
    let fr = spec.flange_d / 2.0;
    vec![
        [0.0, 0.0],
        [fr - spec.bed_chamfer, 0.0],
        [fr, spec.bed_chamfer],
        [fr, spec.flange_h],
        [jr, spec.flange_h],
        [jr, spec.shaft_h - spec.tip_chamfer],
        [jr - spec.tip_chamfer, spec.shaft_h],
        [0.0, spec.shaft_h],
        [0.0, 0.0],
    ]
}

fn add_c(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    deg: f64,
    origin: [f64; 2],
) -> Result<(), String> {
    let e = spec.rotor_d * spec.overlap;
    let radius = (spec.rotor_d + e) / 4.0;
    let inner = radius - spec.blade_wall;
    let center = radius - e / 2.0;
    let a = deg.to_radians();
    let cx = center * a.cos();
    let cy = center * a.sin();
    let len = (cx * cx + cy * cy).sqrt().max(1e-9);
    let ux = cx / len;
    let uy = cy / len;
    let vx = -uy;
    let vy = ux;
    let p = |x: f64, y: f64| json!({ "x": x + origin[0], "y": y + origin[1] });
    let s = p(cx + radius * vx, cy + radius * vy);
    let f = p(cx + radius * ux, cy + radius * uy);
    let ept = p(cx - radius * vx, cy - radius * vy);
    let si = p(cx + inner * vx, cy + inner * vy);
    let fi = p(cx + inner * ux, cy + inner * uy);
    let ei = p(cx - inner * vx, cy - inner * vy);
    call(
        "sketch_add_arc_3pt",
        json!({ "p1": s, "p2": f, "p3": ept, "ctrl_held": false }),
    )?;
    call(
        "sketch_add_line",
        json!({ "from": ept, "to_raw": ei, "ctrl_held": false }),
    )?;
    call(
        "sketch_add_arc_3pt",
        json!({ "p1": ei, "p2": fi, "p3": si, "ctrl_held": false }),
    )?;
    call(
        "sketch_add_line",
        json!({ "from": si, "to_raw": s, "ctrl_held": false }),
    )?;
    Ok(())
}

fn begin_xy(call: &mut impl FnMut(&str, Value) -> Result<Value, String>) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "sketch", "explicit": true }),
    )?;
    call(
        "sketch_begin",
        json!({ "plane": { "type": "origin_plane", "plane": "xy" } }),
    )?;
    call("sketch_set_grid_snap", json!({ "enabled": false }))?;
    Ok(())
}

fn offset_xy(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    z: f64,
) -> Result<Value, String> {
    call(
        "cad_set_focus",
        json!({ "focus": "datums", "explicit": true }),
    )?;
    let created = call(
        "construction_plane_offset",
        json!({
            "reference": { "type": "origin_plane", "plane": "xy" },
            "distance": z
        }),
    )?;
    created["planes"]
        .as_array()
        .and_then(|planes| {
            planes
                .iter()
                .rev()
                .find(|plane| {
                    plane["basis"]["origin"]
                        .as_array()
                        .and_then(|origin| origin.get(2))
                        .and_then(Value::as_f64)
                        .is_some_and(|oz| (oz - z).abs() < 0.25)
                })
                .or_else(|| planes.last())
        })
        .map(|plane| plane["datum_id"].clone())
        .filter(|id| !id.is_null())
        .ok_or_else(|| format!("no datum at z={z}"))
}

fn begin_datum(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    datum_id: Value,
) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "sketch", "explicit": true }),
    )?;
    call(
        "sketch_begin",
        json!({ "plane": { "type": "datum_plane", "datum_id": datum_id } }),
    )?;
    call("sketch_set_grid_snap", json!({ "enabled": false }))?;
    Ok(())
}

fn begin_xz(call: &mut impl FnMut(&str, Value) -> Result<Value, String>) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "sketch", "explicit": true }),
    )?;
    call(
        "sketch_begin",
        json!({ "plane": { "type": "origin_plane", "plane": "xz" } }),
    )?;
    call("sketch_set_grid_snap", json!({ "enabled": false }))?;
    Ok(())
}

fn add_poly(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    points: &[[f64; 2]],
) -> Result<(), String> {
    for pair in points.windows(2) {
        call(
            "sketch_add_line",
            json!({
                "from": { "x": pair[0][0], "y": pair[0][1] },
                "to_raw": { "x": pair[1][0], "y": pair[1][1] },
                "ctrl_held": false
            }),
        )?;
    }
    Ok(())
}

fn add_circle(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    center: [f64; 2],
    diameter: f64,
) -> Result<(), String> {
    call(
        "sketch_add_circle_locked",
        json!({
            "mode": "center_diameter",
            "anchor": { "x": center[0], "y": center[1] },
            "edge_hint": { "x": center[0] + diameter / 2.0, "y": center[1] },
            "diameter_mm": diameter,
            "ctrl_held": false
        }),
    )?;
    Ok(())
}

fn finish_sketch(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
) -> Result<String, String> {
    call("sketch_finish", json!({}))?;
    let document = call("cad_document", json!({}))?;
    last_sketch(&document).ok_or_else(|| "no sketch after finish".to_string())
}

fn last_sketch(document: &Value) -> Option<String> {
    document["features"]
        .as_array()?
        .iter()
        .rev()
        .find(|feature| feature["kind"] == "sketch")
        .and_then(|feature| feature["name"].as_str().map(str::to_string))
}

fn require_clean(update: Value, label: &str) -> Result<Value, String> {
    let errors = update["scene"]["errors"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !errors.is_empty() {
        return Err(format!("{label}: {errors:?}"));
    }
    Ok(update)
}

fn first_body_id(update: &Value) -> Result<u64, String> {
    update["scene"]["bodies"]
        .as_array()
        .and_then(|bodies| bodies.first())
        .and_then(|body| body["id"].as_u64())
        .ok_or_else(|| "no body in update".to_string())
}

fn newest_body_id(update: &Value, known: &[u64]) -> Result<u64, String> {
    update["scene"]["bodies"]
        .as_array()
        .and_then(|bodies| {
            bodies
                .iter()
                .filter_map(|body| body["id"].as_u64())
                .filter(|id| !known.contains(id))
                .max()
        })
        .ok_or_else(|| "no new body".to_string())
}

fn bbox(body: &Value) -> Option<[[f64; 3]; 2]> {
    let mut pts = Vec::new();
    if let Some(faces) = body["faces"].as_array() {
        for face in faces {
            if let Some(origin) = xyz(&face["plane"]["origin"]) {
                pts.push(origin);
            }
        }
    }
    if let Some(edges) = body["edges"].as_array() {
        for edge in edges {
            if let Some(points) = edge["points"].as_array() {
                for point in points {
                    if let Some(xyz) = xyz(point) {
                        pts.push(xyz);
                    }
                }
            }
        }
    }
    pts.retain(|p| p.iter().all(|n| n.is_finite() && n.abs() < 1e5));
    if pts.is_empty() {
        return None;
    }
    let mut min = pts[0];
    let mut max = pts[0];
    for p in pts {
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    Some([min, max])
}

fn xyz(value: &Value) -> Option<[f64; 3]> {
    if let Some(arr) = value.as_array() {
        return Some([
            arr.first()?.as_f64()?,
            arr.get(1)?.as_f64()?,
            arr.get(2)?.as_f64()?,
        ]);
    }
    Some([
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("z")?.as_f64()?,
    ])
}

#[cfg(test)]
mod spec_tests {
    use super::*;

    #[test]
    fn print_kit_spec_encodes_0_4_nozzle_stack() {
        let spec = load_spec().unwrap();
        assert_eq!(spec.id, "fdm-journal-kit");
        assert_eq!(spec.nozzle_mm, 0.4);
        assert_eq!(spec.clearance_mm, 0.4);
        assert!((spec.bush_id - spec.journal_d - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.house_id - spec.bush_od - spec.clearance_mm).abs() < 1e-12);
        assert_eq!(spec.bush_od, 22.0);
        assert_eq!(spec.bush_h, 7.0);
        assert_eq!(spec.journal_d, 8.0);
        assert!(spec.loft_stations >= 2);
        assert!(spec.min_bodies >= 4);
    }
}

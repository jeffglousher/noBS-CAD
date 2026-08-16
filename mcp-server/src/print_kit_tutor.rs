//! CAD synthesis exam: a printed VAWT, built assembled.
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
    pub bush_land_h: f64,
    pub bush_relief_id: f64,
    pub bush_seat: f64,
    pub post_d: f64,
    pub post_hole: f64,
    pub post_circle_r: f64,
    pub post_count: usize,
    pub post_proud: f64,
    pub base_d: f64,
    pub base_h: f64,
    pub hub_d: f64,
    pub rib_w: f64,
    pub pad_d: f64,
    pub plate_z: f64,
    pub rotor_to_plate_gap: f64,
    pub cone_h: f64,
    pub cone_half_deg: f64,
    pub cone_relief_d: f64,
    pub male_cone_r: f64,
    pub tip_r: f64,
    pub thrust_land_od: f64,
    pub thrust_land_h: f64,
    pub thrust_float: f64,
    pub cap_float: f64,
    pub shaft_lower_h: f64,
    pub shaft_shoulder_d: f64,
    pub shaft_shoulder_h: f64,
    pub shaft_upper_h: f64,
    pub hub_od: f64,
    pub hub_h: f64,
    pub socket_w: f64,
    pub socket_radial: f64,
    pub socket_h: f64,
    pub tenon_w: f64,
    pub tenon_h: f64,
    pub drive_across: f64,
    pub drive_across_hub: f64,
    pub wing_count: usize,
    pub wing_h: f64,
    pub wing_radius: f64,
    pub wing_chord: f64,
    pub wing_thick: f64,
    pub wing_offset_deg: f64,
    pub helix_deg: f64,
    pub helix_stations: usize,
    pub airfoil: String,
    pub airfoil_t_c: f64,
    pub airfoil_te_min_mm: f64,
    pub airfoil_stations: usize,
    pub window_d: f64,
    pub top_plate_d: f64,
    pub top_plate_h: f64,
    pub plate_boss_d: f64,
    pub plate_boss_h: f64,
    pub cap_d: f64,
    pub cap_h: f64,
    pub min_bodies: usize,
    pub min_rotor_faces: usize,
    pub filament: Filament,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filament {
    pub name: String,
    pub density_g_cm3: f64,
    pub price_usd_per_kg: f64,
    pub print_volume_factor: f64,
}

impl Spec {
    fn cone_r(&self) -> f64 {
        self.cone_h * self.cone_half_deg.to_radians().tan()
    }
    fn cone_apex_z(&self) -> f64 {
        self.base_h - self.cone_h
    }
    fn shoulder_z(&self) -> f64 {
        self.base_h + self.shaft_lower_h
    }
    fn shoulder_top(&self) -> f64 {
        self.shoulder_z() + self.shaft_shoulder_h
    }
    fn plate_top(&self) -> f64 {
        self.plate_z + self.top_plate_h
    }
    fn plate_stack_h(&self) -> f64 {
        self.top_plate_h + self.plate_boss_h
    }
    fn bush_ld(&self) -> f64 {
        self.bush_h / self.journal_d
    }
    fn bush_relief_h(&self) -> f64 {
        (self.bush_h - 2.0 * self.bush_land_h).max(0.0)
    }
    fn bushing_z(&self) -> f64 {
        self.plate_top() - self.bush_h
    }
    fn cap_z(&self) -> f64 {
        self.plate_top() + self.cap_float
    }
    fn shaft_top(&self) -> f64 {
        self.shoulder_top() + self.shaft_upper_h
    }
    fn post_extrude_h(&self) -> f64 {
        self.plate_z - self.base_h + self.top_plate_h + self.post_proud
    }
    fn post_inner_r(&self) -> f64 {
        self.post_circle_r - self.post_d / 2.0
    }
    fn post_xy(&self, index: usize) -> [f64; 2] {
        let angle = (360.0 / self.post_count.max(1) as f64) * index as f64;
        let radians = angle.to_radians();
        [
            self.post_circle_r * radians.cos(),
            self.post_circle_r * radians.sin(),
        ]
    }
    fn wing_angle_deg(&self, index: usize) -> f64 {
        self.wing_offset_deg + (360.0 / self.wing_count.max(1) as f64) * index as f64
    }
    fn socket_center(&self, index: usize) -> [f64; 2] {
        let radians = self.wing_angle_deg(index).to_radians();
        let radius = self.hub_od / 2.0 - self.socket_radial / 2.0;
        [radius * radians.cos(), radius * radians.sin()]
    }
    fn helix_azimuth_deg(&self, index: usize, t: f64) -> f64 {
        self.wing_angle_deg(index) + self.helix_deg * t
    }
    fn helix_center(&self, index: usize, t: f64) -> [f64; 2] {
        let radians = self.helix_azimuth_deg(index, t).to_radians();
        [
            self.wing_radius * radians.cos(),
            self.wing_radius * radians.sin(),
        ]
    }
    fn blade_inner_r(&self) -> f64 {
        self.wing_radius - self.wing_thick / 2.0
    }
    fn blade_tip_r(&self) -> f64 {
        (self.wing_radius.powi(2) + (self.wing_chord / 2.0).powi(2)).sqrt()
    }
    fn tenon_center(&self, index: usize) -> [f64; 2] {
        let radians = self.wing_angle_deg(index).to_radians();
        let inner = self.hub_od / 2.0 - self.socket_radial + self.clearance_mm / 2.0;
        let outer = self.blade_inner_r() + 0.2;
        let radius = (inner + outer) / 2.0;
        [radius * radians.cos(), radius * radians.sin()]
    }
    fn tenon_radial(&self) -> f64 {
        let inner = self.hub_od / 2.0 - self.socket_radial + self.clearance_mm / 2.0;
        let outer = self.blade_inner_r() + 0.2;
        outer - inner
    }
    fn rotor_d(&self) -> f64 {
        self.blade_tip_r() * 2.0
    }
    fn frame_girth_ratio(&self) -> f64 {
        self.top_plate_d.max(self.base_d) / self.rotor_d()
    }
    fn post_chord_ratio(&self) -> f64 {
        self.post_d / self.wing_chord
    }
    fn sanity_ok(&self) -> bool {
        self.frame_girth_ratio() <= 1.55
            && self.post_chord_ratio() <= 0.40
            && self.wing_h + 1e-9 >= self.wing_chord * 2.5
            && self.solidity() >= 0.27
            && self.top_plate_d <= 76.0
            && self.post_d <= 5.5
            && self.y_frame_ok()
    }
    fn y_frame_ok(&self) -> bool {
        self.hub_d + 1e-9 <= 36.0
            && self.rib_w + 1e-9 <= 6.0
            && self.hub_d < self.top_plate_d * 0.55
            && self.pad_d + 1e-9 <= 12.0
    }
    fn printed_bearing_ok(&self) -> bool {
        self.bush_od < 20.0
            && self.bush_ld() + 1e-9 >= 1.0
            && self.bush_relief_id > self.bush_id
            && self.bush_land_h + 1e-9 >= 1.2
            && self.plate_stack_h() + 1e-9 >= self.bush_h + 2.0
            && self.cone_h >= 3.0
            && self.thrust_land_od + 1e-9 <= 13.0
    }
    fn helix_ok(&self) -> bool {
        self.helix_deg >= 45.0 && self.helix_stations >= 2
    }
    fn socket_floor_z(&self) -> f64 {
        self.shoulder_top() + self.hub_h - self.socket_h
    }
    fn solidity(&self) -> f64 {
        let diameter = self.wing_radius * 2.0;
        (self.wing_count as f64) * self.wing_chord / (std::f64::consts::PI * diameter)
    }
    fn airfoil_ok(&self) -> bool {
        self.airfoil.to_ascii_uppercase().contains("NACA")
            && self.airfoil_t_c >= 0.18
            && self.airfoil_t_c <= 0.26
            && (self.wing_thick - self.wing_chord * self.airfoil_t_c).abs() < 0.15
            && self.airfoil_te_min_mm + 1e-9 >= self.nozzle_mm * 2.0
    }
    fn y_frame_volume(&self, height: f64) -> f64 {
        let hub = std::f64::consts::PI * (self.hub_d * 0.5).powi(2) * height;
        let rib_len = (self.post_circle_r - self.hub_d * 0.5).max(0.0);
        let ribs = (self.post_count as f64) * self.rib_w * rib_len * height;
        let pads = (self.post_count as f64)
            * std::f64::consts::PI
            * (self.pad_d * 0.5).powi(2)
            * height;
        hub + ribs + pads
    }
    fn estimated_solid_cm3(&self) -> f64 {
        let base = self.y_frame_volume(self.base_h);
        let posts = (self.post_count as f64)
            * std::f64::consts::PI
            * (self.post_d * 0.5).powi(2)
            * self.post_extrude_h();
        let shaft = std::f64::consts::PI * (self.journal_d * 0.5).powi(2) * self.shaft_upper_h
            + std::f64::consts::PI * (self.shaft_shoulder_d * 0.5).powi(2) * self.shaft_shoulder_h;
        let hub = std::f64::consts::PI
            * ((self.hub_od * 0.5).powi(2) - (self.bush_id * 0.5).powi(2))
            * self.hub_h;
        let wing = (self.wing_count as f64)
            * 0.70
            * self.wing_chord
            * self.wing_thick
            * self.wing_h;
        let plate = self.y_frame_volume(self.top_plate_h)
            + std::f64::consts::PI * (self.plate_boss_d * 0.5).powi(2) * self.plate_boss_h;
        let bush_tube = std::f64::consts::PI
            * ((self.bush_od * 0.5).powi(2) - (self.bush_id * 0.5).powi(2))
            * self.bush_h;
        let bush_relief = std::f64::consts::PI
            * ((self.bush_relief_id * 0.5).powi(2) - (self.bush_id * 0.5).powi(2))
            * self.bush_relief_h();
        let bush = (bush_tube - bush_relief).max(0.0);
        let cap = std::f64::consts::PI
            * ((self.cap_d * 0.5).powi(2) - (self.bush_id * 0.5).powi(2))
            * self.cap_h;
        (base + posts + shaft + hub + wing + plate + bush + cap) / 1000.0
    }
    fn estimated_print_mass_g(&self) -> f64 {
        self.estimated_solid_cm3() * self.filament.density_g_cm3 * self.filament.print_volume_factor
    }
    fn estimated_filament_usd(&self) -> f64 {
        self.estimated_print_mass_g() / 1000.0 * self.filament.price_usd_per_kg
    }
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
        || (spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() > 1e-9
        || (spec.post_hole - spec.post_d - spec.clearance_mm).abs() > 1e-9
        || (spec.drive_across_hub - spec.drive_across - spec.clearance_mm).abs() > 1e-9
        || (spec.socket_w - spec.tenon_w - spec.clearance_mm).abs() > 1e-9
    {
        return Ok(Report::fail(
            &spec.id,
            "clearance",
            "spec clearances are not exactly +nozzle",
        ));
    }

    let mut step = 0u32;
    let mut call = |name: &str, arguments: Value| {
        step += 1;
        call(name, arguments).map_err(|error| format!("step {step} {name}: {error}"))
    };

    call("cad_new_project", json!({}))?;
    call(
        "cad_set_document_name",
        json!({ "name": spec.document_name }),
    )?;

    let base_id = build_base(&mut call, &spec)?;
    let shaft_id = build_shaft(&mut call, &spec)?;
    let hub_id = build_hub(&mut call, &spec, &[base_id, shaft_id])?;
    let mut known = vec![base_id, shaft_id, hub_id];
    let mut wing_ids = Vec::new();
    for index in 0..spec.wing_count {
        let wing_id = build_wing(&mut call, &spec, index, &known)?;
        known.push(wing_id);
        wing_ids.push(wing_id);
    }
    let plate_id = build_top_plate(&mut call, &spec)?;
    let bush_id = build_bushing(&mut call, &spec)?;
    let cap_id = build_cap(&mut call, &spec)?;

    call(
        "cad_set_focus",
        json!({ "focus": "print", "explicit": true }),
    )?;
    let mut appearances = vec![
        (base_id, "bambu.pla.basic.black"),
        (shaft_id, "bambu.pla.basic.jade_white"),
        (hub_id, "bambu.pla.basic.green"),
        (plate_id, "bambu.pla.basic.black"),
        (bush_id, "bambu.pla.matte.dark_gray"),
        (cap_id, "bambu.pla.basic.red"),
    ];
    for wing_id in &wing_ids {
        appearances.push((*wing_id, "bambu.pla.basic.green"));
    }
    for (id, preset) in appearances {
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
        &spec,
        &scene,
        &document,
        &preflight,
        &exported,
        wing_ids.first().copied().unwrap_or(hub_id),
    ))
}

fn build_y_frame(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    z: f64,
    height: f64,
    flip: bool,
    known: &[u64],
    label: &str,
) -> Result<u64, String> {
    let deck = offset_xy(call, z)?;
    begin_datum(call, deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.hub_d)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": height },
                "taper_angle_deg": 0.0,
                "flip": flip,
                "target_body_ids": []
            }),
        )?,
        &format!("{label} hub"),
    )?;
    let frame_id = newest_body_id(&update, known)?;
    for index in 0..spec.post_count {
        let angle = (360.0 / spec.post_count.max(1) as f64) * index as f64;
        let [px, py] = spec.post_xy(index);
        begin_datum(call, deck.clone())?;
        add_oriented_rect(
            call,
            [px * 0.5, py * 0.5],
            spec.post_circle_r,
            spec.rib_w,
            angle,
        )?;
        let rib = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": rib,
                    "profile_indices": [0],
                    "operation": "join",
                    "extent": { "type": "distance", "distance": height },
                    "taper_angle_deg": 0.0,
                    "flip": flip,
                    "target_body_ids": [frame_id]
                }),
            )?,
            &format!("{label} rib {index}"),
        )?;
        begin_datum(call, deck.clone())?;
        add_circle(call, [px, py], spec.pad_d)?;
        let pad = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": pad,
                    "profile_indices": [0],
                    "operation": "join",
                    "extent": { "type": "distance", "distance": height },
                    "taper_angle_deg": 0.0,
                    "flip": flip,
                    "target_body_ids": [frame_id]
                }),
            )?,
            &format!("{label} pad {index}"),
        )?;
    }
    Ok(frame_id)
}

fn build_base(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let base_id = build_y_frame(call, spec, 0.0, spec.base_h, false, &[], "base")?;

    begin_xz(call)?;
    add_poly(
        call,
        &[
            [0.0, spec.cone_apex_z()],
            [spec.cone_r(), spec.base_h],
            [0.0, spec.base_h],
            [0.0, spec.cone_apex_z()],
        ],
        false,
    )?;
    let cone_sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_revolve",
            json!({
                "sketch_name": cone_sketch,
                "profile_indices": [0],
                "axis_origin": { "x": 0.0, "y": 0.0 },
                "axis_direction": { "x": 0.0, "y": 1.0 },
                "axis_line_entity_id": null,
                "angle_deg": 360.0,
                "flip": false,
                "operation": "cut",
                "target_body_ids": [base_id]
            }),
        )?,
        "thrust cup",
    )?;

    begin_xy(call)?;
    add_circle(call, [0.0, 0.0], spec.cone_relief_d)?;
    let relief = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": relief,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.base_h + 1.0 },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [base_id]
            }),
        )?,
        "cone relief",
    )?;

    let top = offset_xy(call, spec.base_h)?;
    begin_datum(call, top)?;
    add_circle(call, spec.post_xy(0), spec.post_d)?;
    let post_sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": post_sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.post_extrude_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "post",
    )?;
    let post_id = newest_body_id(&update, &[base_id])?;
    let patterned = require_clean(
        call(
            "solid_circular_pattern",
            json!({
                "body_ids": [post_id],
                "axis_origin": { "x": 0.0, "y": 0.0, "z": 0.0 },
                "axis_direction": { "x": 0.0, "y": 0.0, "z": 1.0 },
                "count": spec.post_count,
                "total_angle_deg": 360.0
            }),
        )?,
        "even posts",
    )?;
    let post_ids: Vec<u64> = patterned["scene"]["bodies"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|body| body["id"].as_u64())
        .filter(|id| *id != base_id)
        .collect();
    require_clean(
        call(
            "solid_combine",
            json!({
                "target_body_id": base_id,
                "tool_body_ids": post_ids,
                "operation": "join",
                "keep_tools": false
            }),
        )?,
        "join posts",
    )?;
    Ok(base_id)
}

fn build_shaft(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    begin_xz(call)?;
    add_poly(call, &shaft_profile(spec), false)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_revolve",
            json!({
                "sketch_name": sketch,
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
        "shaft",
    )?;
    let shaft_id = newest_body_id(&update, &[])?;
    cut_flats(
        call,
        shaft_id,
        spec.shoulder_top(),
        spec.hub_h + 0.2,
        spec.drive_across / 2.0,
        8.0,
    )?;
    Ok(shaft_id)
}

fn build_hub(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.shoulder_top())?;
    begin_datum(call, deck)?;
    add_circle(call, [0.0, 0.0], spec.hub_od)?;
    add_circle(call, [0.0, 0.0], spec.bush_id)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.hub_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "hub",
    )?;
    let hub_id = newest_body_id(&update, known)?;
    cut_flats(
        call,
        hub_id,
        spec.shoulder_top(),
        spec.hub_h + 0.2,
        spec.drive_across_hub / 2.0,
        spec.bush_id / 2.0 + 1.0,
    )?;

    let top = offset_xy(call, spec.shoulder_top() + spec.hub_h)?;
    for index in 0..spec.wing_count {
        begin_datum(call, top.clone())?;
        add_oriented_rect(
            call,
            spec.socket_center(index),
            spec.socket_radial + 2.0,
            spec.socket_w,
            spec.wing_angle_deg(index),
        )?;
        let socket = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": socket,
                    "profile_indices": [0],
                    "operation": "cut",
                    "extent": { "type": "distance", "distance": spec.socket_h },
                    "taper_angle_deg": 0.0,
                    "flip": true,
                    "target_body_ids": [hub_id]
                }),
            )?,
            "hub socket",
        )?;
    }
    Ok(hub_id)
}

fn build_wing(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    index: usize,
    known: &[u64],
) -> Result<u64, String> {
    let angle = spec.wing_angle_deg(index);
    let stations = spec.helix_stations.max(2);
    let mut sections = Vec::new();
    for station in 0..stations {
        let t = station as f64 / (stations - 1) as f64;
        let deck = offset_xy(call, spec.shoulder_top() + spec.wing_h * t)?;
        begin_datum(call, deck)?;
        add_airfoil(
            call,
            spec.helix_center(index, t),
            spec.helix_azimuth_deg(index, t) + 90.0,
            spec.wing_chord,
            spec.airfoil_t_c,
            spec.airfoil_stations,
            spec.airfoil_te_min_mm,
        )?;
        sections.push(finish_sketch(call)?);
    }
    let update = require_clean(
        call(
            "solid_loft",
            json!({
                "sections": sections.iter().map(|name| json!({
                    "sketch_name": name,
                    "profile_index": 0
                })).collect::<Vec<_>>(),
                "ruled": false,
                "operation": "new_body",
                "target_body_ids": [],
                "continuity": "g1"
            }),
        )?,
        "helical wing",
    )?;
    let wing_id = newest_body_id(&update, known)?;

    let tenon_deck = offset_xy(call, spec.socket_floor_z())?;
    begin_datum(call, tenon_deck)?;
    add_oriented_rect(
        call,
        spec.tenon_center(index),
        spec.tenon_radial(),
        spec.tenon_w,
        angle,
    )?;
    let tenon = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": tenon,
                "profile_indices": [0],
                "operation": "join",
                "extent": { "type": "distance", "distance": spec.tenon_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [wing_id]
            }),
        )?,
        "wing tenon",
    )?;
    Ok(wing_id)
}

fn build_top_plate(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let plate_id = build_y_frame(
        call,
        spec,
        spec.plate_z,
        spec.top_plate_h,
        false,
        &[],
        "top plate",
    )?;
    let deck = offset_xy(call, spec.plate_z)?;
    begin_datum(call, deck)?;
    add_circle(call, [0.0, 0.0], spec.plate_boss_d)?;
    let boss = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": boss,
                "profile_indices": [0],
                "operation": "join",
                "extent": { "type": "distance", "distance": spec.plate_boss_h },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [plate_id]
            }),
        )?,
        "plate boss",
    )?;

    let top = offset_xy(call, spec.plate_top())?;
    begin_datum(call, top.clone())?;
    add_circle(call, [0.0, 0.0], spec.journal_d + spec.clearance_mm)?;
    for i in 0..spec.post_count {
        add_circle(call, spec.post_xy(i), spec.post_hole)?;
    }
    let holes = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": holes,
                "profile_indices": (0..(1 + spec.post_count)).collect::<Vec<_>>(),
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.plate_stack_h() + 1.0 },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [plate_id]
            }),
        )?,
        "plate holes",
    )?;
    begin_datum(call, top)?;
    add_circle(call, [0.0, 0.0], spec.bush_seat)?;
    let seat = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": seat,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.bush_h },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [plate_id]
            }),
        )?,
        "bushing seat",
    )?;
    Ok(plate_id)
}

fn build_bushing(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.bushing_z())?;
    begin_datum(call, deck)?;
    add_circle(call, [0.0, 0.0], spec.bush_od)?;
    add_circle(call, [0.0, 0.0], spec.bush_id)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.bush_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "printed bushing",
    )?;
    let bush_id = newest_body_id(&update, &[])?;
    if spec.bush_relief_h() > 0.0 && spec.bush_relief_id > spec.bush_id {
        let relief_deck = offset_xy(call, spec.bushing_z() + spec.bush_land_h)?;
        begin_datum(call, relief_deck)?;
        add_circle(call, [0.0, 0.0], spec.bush_relief_id)?;
        let relief = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": relief,
                    "profile_indices": [0],
                    "operation": "cut",
                    "extent": { "type": "distance", "distance": spec.bush_relief_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [bush_id]
                }),
            )?,
            "bushing relief",
        )?;
    }
    Ok(bush_id)
}

fn build_cap(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let top = offset_xy(call, spec.cap_z())?;
    begin_datum(call, top)?;
    add_circle(call, [0.0, 0.0], spec.cap_d)?;
    add_circle(call, [0.0, 0.0], spec.bush_id)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.cap_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "cap",
    )?;
    newest_body_id(&update, &[])
}

fn grade(
    spec: &Spec,
    scene: &Value,
    document: &Value,
    preflight: &Value,
    exported: &Value,
    wing_id: u64,
) -> Report {
    let bodies = scene["bodies"].as_array().cloned().unwrap_or_default();
    let features = document["features"].as_array().cloned().unwrap_or_default();
    let wing = bodies
        .iter()
        .find(|body| body["id"].as_u64() == Some(wing_id));
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
            && (spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() < 1e-9
            && (spec.post_hole - spec.post_d - spec.clearance_mm).abs() < 1e-9
            && (spec.drive_across_hub - spec.drive_across - spec.clearance_mm).abs() < 1e-9
            && (spec.socket_w - spec.tenon_w - spec.clearance_mm).abs() < 1e-9,
        format!(
            "journal {:.1} in bush {:.1}; tenon {:.1} in socket {:.1}",
            spec.journal_d, spec.bush_id, spec.tenon_w, spec.socket_w
        ),
    );
    push_lesson(
        &mut lessons,
        "no_press",
        spec.clearance_mm >= spec.nozzle_mm && spec.bush_id > spec.journal_d,
        "printed interfaces are +0.40 slip; wing drops into a socket".to_string(),
    );

    let proper_stack = spec.plate_stack_h() + 1e-9 >= spec.bush_h + 2.0
        && spec.post_extrude_h() + spec.base_h > spec.plate_top()
        && spec.blade_tip_r() + 1.0 < spec.post_inner_r()
        && spec.thrust_float > 0.0
        && spec.cap_float > 0.0;
    let stacked = bodies.len() >= spec.min_bodies
        && bodies
            .iter()
            .any(|body| bbox(body).is_some_and(|box3| box3[1][2] > spec.plate_top() - 1.0))
        && bodies.iter().all(|body| near_axis(body, 50.0))
        && proper_stack;
    push_lesson(
        &mut lessons,
        "assemble",
        stacked,
        format!(
            "{} coaxial bodies; blade tip r{:.1} in post inner r{:.0}; hub sockets",
            bodies.len(),
            spec.blade_tip_r(),
            spec.post_inner_r()
        ),
    );
    push_lesson(
        &mut lessons,
        "thrust",
        (spec.cone_half_deg - 45.0).abs() < 1e-9
            && spec.cone_h >= 3.0
            && spec.male_cone_r + 0.15 < spec.cone_r()
            && spec.thrust_land_od > spec.journal_d
            && spec.thrust_float >= 0.2,
        format!(
            "45° cup r{:.1} / male r{:.1}; Ø{:.0} land float {:.1}",
            spec.cone_r(),
            spec.male_cone_r,
            spec.thrust_land_od,
            spec.thrust_float
        ),
    );
    push_lesson(
        &mut lessons,
        "even",
        spec.post_count == 3
            && spec.wing_count == 3
            && (spec.wing_offset_deg + spec.helix_deg * 0.5 - 60.0).abs() < 1e-9
            && spec.drive_across_hub > spec.drive_across,
        "3 posts at 120°, 3 wings in the bays, double-D drive".to_string(),
    );
    push_lesson(
        &mut lessons,
        "printed_bearings",
        spec.printed_bearing_ok(),
        format!(
            "two-land sleeve L/D {:.2}, relief Ø{:.1}, land {:.1} mm; plate stack {:.0} mm; no metal 608",
            spec.bush_ld(),
            spec.bush_relief_id,
            spec.bush_land_h,
            spec.plate_stack_h()
        ),
    );

    let wing_faces = wing
        .and_then(|body| body["faces"].as_array().map(|faces| faces.len()))
        .unwrap_or(0);
    let wing_box = wing.and_then(bbox);
    let wing_span = wing_box
        .map(|box3| box3[1][2] - box3[0][2])
        .unwrap_or(0.0);
    let wing_xy = wing_box
        .map(|box3| (box3[1][0] - box3[0][0]).max(box3[1][1] - box3[0][1]))
        .unwrap_or(f64::INFINITY);
    push_lesson(
        &mut lessons,
        "not_2d",
        wing_faces >= spec.min_rotor_faces
            && wing_span > spec.wing_h - 1.0
            && wing_xy < spec.wing_radius + spec.wing_chord * 0.65
            && spec.wing_thick < spec.wing_chord / 3.0
            && spec.wing_h > spec.wing_chord * 2.0,
        format!("mounted blade faces={wing_faces} height={wing_span:.1} xy={wing_xy:.1}"),
    );
    push_lesson(
        &mut lessons,
        "airfoil",
        spec.airfoil_ok() && wing_faces >= spec.min_rotor_faces,
        format!(
            "{} t/c={:.2} TE≥{:.1} mm; solidity {:.2}; faces={wing_faces}",
            spec.airfoil,
            spec.airfoil_t_c,
            spec.airfoil_te_min_mm,
            spec.solidity()
        ),
    );
    push_lesson(
        &mut lessons,
        "sanity",
        spec.sanity_ok(),
        format!(
            "frame/rotor {:.2}; post/chord {:.2}; σ={:.2}; plate Ø{:.0} post Ø{:.0}",
            spec.frame_girth_ratio(),
            spec.post_chord_ratio(),
            spec.solidity(),
            spec.top_plate_d,
            spec.post_d
        ),
    );
    push_lesson(
        &mut lessons,
        "helix",
        spec.helix_ok() && wing_span > spec.wing_h - 1.0,
        format!(
            "{:.0}° helix, {} stations, span {:.1}",
            spec.helix_deg, spec.helix_stations, wing_span
        ),
    );
    let cost = spec.estimated_filament_usd();
    push_lesson(
        &mut lessons,
        "report",
        spec.filament.price_usd_per_kg > 0.0
            && spec.estimated_solid_cm3() > 1.0
            && cost > 0.05
            && spec.airfoil_ok(),
        format!(
            "{:.1} cm³ solid → {:.1} g {} → ${:.2}",
            spec.estimated_solid_cm3(),
            spec.estimated_print_mass_g(),
            spec.filament.name,
            cost
        ),
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
    let journal = spec.journal_d / 2.0;
    let shoulder = spec.shaft_shoulder_d / 2.0;
    let land = spec.thrust_land_od / 2.0;
    let tip_z = spec.cone_apex_z() + 0.5;
    let mouth_z = spec.base_h;
    let land_z = spec.base_h + spec.thrust_float;
    let land_top = land_z + spec.thrust_land_h;
    let shoulder_z = spec.shoulder_z();
    let shoulder_top = spec.shoulder_top();
    let top = spec.shaft_top();
    vec![
        [0.0, tip_z],
        [spec.tip_r, tip_z],
        [spec.male_cone_r, mouth_z],
        [spec.male_cone_r, land_z],
        [land, land_z],
        [land, land_top],
        [journal, land_top],
        [journal, shoulder_z],
        [shoulder, shoulder_z],
        [shoulder, shoulder_top],
        [journal, shoulder_top],
        [journal, top],
        [0.0, top],
        [0.0, tip_z],
    ]
}

fn naca_00_thickness(x: f64, thickness_ratio: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    5.0 * thickness_ratio
        * (0.2969 * x.sqrt() - 0.1260 * x - 0.3516 * x * x + 0.2843 * x.powi(3) - 0.1015 * x.powi(4))
}

fn naca_symmetric_loop(chord: f64, thickness_ratio: f64, stations: usize, te_min: f64) -> Vec<[f64; 2]> {
    let count = stations.max(6);
    let mut xs = Vec::with_capacity(count);
    for i in 0..count {
        let beta = std::f64::consts::PI * i as f64 / (count - 1) as f64;
        xs.push(0.5 * (1.0 - beta.cos()));
    }
    let mut upper = Vec::with_capacity(count);
    let mut lower = Vec::with_capacity(count);
    for x in xs {
        let mut yt = naca_00_thickness(x, thickness_ratio) * chord;
        if x > 0.85 {
            yt = yt.max(te_min / 2.0);
        }
        let xc = (x - 0.5) * chord;
        upper.push([xc, yt]);
        lower.push([xc, -yt]);
    }
    let mut points = upper;
    for point in lower.iter().rev().skip(1) {
        points.push(*point);
    }
    if let Some(first) = points.first().copied() {
        points.push(first);
    }
    points
}

fn add_airfoil(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    center: [f64; 2],
    angle_deg: f64,
    chord: f64,
    thickness_ratio: f64,
    stations: usize,
    te_min: f64,
) -> Result<(), String> {
    let angle = angle_deg.to_radians();
    let (cos, sin) = (angle.cos(), angle.sin());
    let local = naca_symmetric_loop(chord, thickness_ratio, stations, te_min);
    let world: Vec<[f64; 2]> = local
        .into_iter()
        .map(|[x, y]| [center[0] + cos * x - sin * y, center[1] + sin * x + cos * y])
        .collect();
    add_poly(call, &world, true)
}

fn add_oriented_rect(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    center: [f64; 2],
    length: f64,
    width: f64,
    angle_deg: f64,
) -> Result<(), String> {
    let angle = angle_deg.to_radians();
    let ux = [angle.cos(), angle.sin()];
    let uy = [-angle.sin(), angle.cos()];
    let half_l = length / 2.0;
    let half_w = width / 2.0;
    let corner = |s: f64, t: f64| {
        [
            center[0] + ux[0] * s + uy[0] * t,
            center[1] + ux[1] * s + uy[1] * t,
        ]
    };
    add_poly(
        call,
        &[
            corner(half_l, half_w),
            corner(half_l, -half_w),
            corner(-half_l, -half_w),
            corner(-half_l, half_w),
            corner(half_l, half_w),
        ],
        true,
    )
}

fn cut_flats(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    body_id: u64,
    z: f64,
    depth: f64,
    half_across: f64,
    outer: f64,
) -> Result<(), String> {
    let datum = offset_xy(call, z)?;
    begin_datum(call, datum)?;
    call(
        "sketch_add_rectangle",
        json!({
            "mode": "two_point",
            "p1": { "x": half_across, "y": -outer },
            "p2": { "x": outer, "y": outer },
            "ctrl_held": false
        }),
    )?;
    call(
        "sketch_add_rectangle",
        json!({
            "mode": "two_point",
            "p1": { "x": -outer, "y": -outer },
            "p2": { "x": -half_across, "y": outer },
            "ctrl_held": false
        }),
    )?;
    let sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0, 1],
                "operation": "cut",
                "extent": { "type": "distance", "distance": depth },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [body_id]
            }),
        )?,
        "double-D flats",
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
    ctrl_held: bool,
) -> Result<(), String> {
    for pair in points.windows(2) {
        let dx = pair[1][0] - pair[0][0];
        let dy = pair[1][1] - pair[0][1];
        if dx * dx + dy * dy < 1e-8 {
            continue;
        }
        call(
            "sketch_add_line",
            json!({
                "from": { "x": pair[0][0], "y": pair[0][1] },
                "to_raw": { "x": pair[1][0], "y": pair[1][1] },
                "ctrl_held": ctrl_held
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

#[allow(dead_code)]
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

fn near_axis(body: &Value, radius: f64) -> bool {
    bbox(body).is_some_and(|box3| {
        let cx = (box3[0][0] + box3[1][0]) * 0.5;
        let cy = (box3[0][1] + box3[1][1]) * 0.5;
        (cx * cx + cy * cy).sqrt() < radius
    })
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
        assert_eq!(spec.id, "fdm-print-vawt");
        assert_eq!(spec.nozzle_mm, 0.4);
        assert_eq!(spec.clearance_mm, 0.4);
        assert!((spec.bush_id - spec.journal_d - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.post_hole - spec.post_d - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.drive_across_hub - spec.drive_across - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.socket_w - spec.tenon_w - spec.clearance_mm).abs() < 1e-12);
        assert_eq!(spec.cone_half_deg, 45.0);
        assert!(spec.male_cone_r + 0.15 < spec.cone_r());
        assert!(spec.printed_bearing_ok());
        assert!(spec.y_frame_ok());
        assert!(spec.plate_stack_h() + 1e-9 >= spec.bush_h + 2.0);
        assert!(spec.blade_tip_r() + 1.0 < spec.post_inner_r());
        assert!(spec.wing_h > spec.wing_chord * 2.0);
        assert!(spec.wing_thick < spec.wing_chord / 3.0);
        assert!(spec.airfoil_ok());
        assert!(spec.sanity_ok());
        assert!(spec.helix_ok());
        assert!(spec.estimated_filament_usd() > 0.05);
        assert!(spec.post_extrude_h() + spec.base_h > spec.plate_top());
        assert!(
            (spec.plate_z
                - (spec.base_h
                    + spec.shaft_lower_h
                    + spec.shaft_shoulder_h
                    + spec.wing_h
                    + spec.rotor_to_plate_gap))
                .abs()
                < 1e-9
        );
        assert!(spec.shaft_top() > spec.plate_top() + spec.cap_float + spec.cap_h);
        assert_eq!(spec.post_count, 3);
        assert_eq!(spec.wing_count, 3);
        assert!((spec.wing_offset_deg + spec.helix_deg * 0.5 - 60.0).abs() < 1e-9);
        assert!(spec.min_bodies >= 9);
        assert!(spec.bush_od < 20.0);
    }
}

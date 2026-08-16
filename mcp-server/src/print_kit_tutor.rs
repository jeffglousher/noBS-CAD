//! CAD synthesis exam: a printed VAWT *assembly*.
//!
//! Spec: `scripts/fixtures/print-kit-tutor.spec.json`.
//! Linear numbers are the X2D-max design; `scale` shrinks the source.

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
    pub scale: f64,
    pub max_scale: f64,
    pub printer: Printer,
    pub fit_running_mm: f64,
    pub fit_slip_mm: f64,
    pub fit_friction_mm: f64,
    pub filament: Filament,
    pub wing_count: usize,
    pub wing_h: f64,
    pub wing_radius: f64,
    pub wing_chord_root: f64,
    pub wing_chord_tip: f64,
    pub wing_thick: f64,
    pub wing_offset_deg: f64,
    pub helix_deg: f64,
    pub helix_stations: usize,
    pub airfoil: String,
    pub airfoil_t_c: f64,
    pub airfoil_te_min_mm: f64,
    pub airfoil_stations: usize,
    pub hub_h: f64,
    pub roller_count: usize,
    pub roller_d: f64,
    pub roller_h: f64,
    pub roller_min_d: f64,
    pub inner_race_d: f64,
    pub axle_flange_d: f64,
    pub axle_flange_h: f64,
    pub axle_square: f64,
    pub axle_square_h: f64,
    pub base_h: f64,
    pub rib_w: f64,
    pub pad_d: f64,
    pub post_count: usize,
    pub post_circle_r: f64,
    pub base_boss_d: f64,
    pub cage_h: f64,
    pub retainer_h: f64,
    pub thrust_float: f64,
    pub min_bodies: usize,
    pub min_rotor_faces: usize,
    pub min_print_plates: usize,
    pub print_plates: Vec<String>,
    pub retired_print_plates: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Printer {
    pub name: String,
    pub bed_mm: [f64; 3],
    pub margin_mm: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filament {
    pub name: String,
    pub density_g_cm3: f64,
    pub price_usd_per_kg: f64,
    pub print_volume_factor: f64,
}

impl Spec {
    fn mm(&self, value: f64) -> f64 {
        value * self.scale
    }
    fn mm_min(&self, value: f64, min: f64) -> f64 {
        (value * self.scale).max(min)
    }
    fn wall(&self) -> f64 {
        self.mm_min(4.0, self.nozzle_mm * 4.0)
    }
    fn te_min(&self) -> f64 {
        self.airfoil_te_min_mm.max(self.nozzle_mm * 2.0)
    }
    fn roller_d(&self) -> f64 {
        self.mm_min(self.roller_d, self.roller_min_d)
    }
    fn roller_h(&self) -> f64 {
        self.mm_min(self.roller_h, 6.0)
    }
    fn inner_race_d(&self) -> f64 {
        self.mm_min(self.inner_race_d, self.roller_d() + 4.0)
    }
    fn hub_bore(&self) -> f64 {
        self.inner_race_d() + 2.0 * self.roller_d() + self.fit_running_mm
    }
    fn hub_od(&self) -> f64 {
        self.hub_bore() + 2.0 * self.wall() * 2.0
    }
    fn hub_h(&self) -> f64 {
        self.mm_min(self.hub_h, 8.0).max(self.roller_h())
    }
    fn hub_square(&self) -> f64 {
        self.mm(self.axle_square) + self.fit_friction_mm
    }
    fn axle_square(&self) -> f64 {
        self.mm(self.axle_square)
    }
    fn axle_square_h(&self) -> f64 {
        self.mm_min(self.axle_square_h, self.hub_h() + 2.0)
    }
    fn axle_flange_d(&self) -> f64 {
        self.mm(self.axle_flange_d).max(self.hub_od() + 6.0)
    }
    fn axle_flange_h(&self) -> f64 {
        self.mm_min(self.axle_flange_h, 2.4)
    }
    fn wing_h(&self) -> f64 {
        self.mm(self.wing_h)
    }
    fn wing_radius(&self) -> f64 {
        self.mm(self.wing_radius)
    }
    fn chord_root(&self) -> f64 {
        self.mm(self.wing_chord_root)
    }
    fn chord_tip(&self) -> f64 {
        self.mm(self.wing_chord_tip)
    }
    fn wing_thick(&self) -> f64 {
        self.chord_root() * self.airfoil_t_c
    }
    fn base_h(&self) -> f64 {
        self.mm_min(self.base_h, 6.0)
    }
    fn rib_w(&self) -> f64 {
        self.mm_min(self.rib_w, 5.0)
    }
    fn pad_d(&self) -> f64 {
        self.mm_min(self.pad_d, 10.0)
    }
    fn post_circle_r(&self) -> f64 {
        self.mm(self.post_circle_r)
    }
    fn base_boss_d(&self) -> f64 {
        self.axle_flange_d()
            .max(self.hub_od() + 8.0)
            .max(self.mm(self.base_boss_d))
    }
    fn cage_od(&self) -> f64 {
        self.hub_bore() - self.fit_running_mm
    }
    fn cage_id(&self) -> f64 {
        self.inner_race_d() + self.fit_slip_mm
    }
    fn cage_h(&self) -> f64 {
        self.mm_min(self.cage_h, self.roller_h() + 2.0)
    }
    fn cage_pocket(&self) -> f64 {
        self.roller_d() + self.fit_running_mm
    }
    fn retainer_od(&self) -> f64 {
        self.axle_flange_d()
            .min(self.hub_od() + 4.0)
            .max(self.hub_bore() + 4.0)
    }
    fn retainer_id(&self) -> f64 {
        self.axle_square() + self.fit_slip_mm
    }
    fn retainer_h(&self) -> f64 {
        self.mm_min(self.retainer_h, 2.0)
    }
    fn pcd(&self) -> f64 {
        (self.inner_race_d() + self.hub_bore()) * 0.5
    }
    fn usable_bed(&self) -> [f64; 3] {
        [
            self.printer.bed_mm[0] - 2.0 * self.printer.margin_mm,
            self.printer.bed_mm[1] - 2.0 * self.printer.margin_mm,
            self.printer.bed_mm[2] - 2.0 * self.printer.margin_mm,
        ]
    }
    fn blade_tip_r(&self) -> f64 {
        self.wing_radius() + self.chord_tip() * 0.15
    }
    fn rotor_d(&self) -> f64 {
        self.blade_tip_r() * 2.0
    }
    fn base_envelope(&self) -> f64 {
        self.post_circle_r() * 2.0 + self.pad_d()
    }
    fn rotor_print_h(&self) -> f64 {
        self.hub_h() + self.wing_h()
    }
    fn fits_x2d_at_max(&self) -> bool {
        let bed = self.usable_bed();
        self.rotor_d() / self.scale <= bed[0]
            && self.rotor_print_h() / self.scale <= bed[2]
            && self.base_envelope() / self.scale <= bed[0]
    }
    fn flange_z(&self) -> f64 {
        self.base_h() + self.thrust_float
    }
    fn race_z(&self) -> f64 {
        self.flange_z() + self.axle_flange_h()
    }
    fn hub_z(&self) -> f64 {
        self.race_z()
    }
    fn retainer_z(&self) -> f64 {
        self.hub_z() + self.hub_h()
    }
    fn post_h(&self) -> f64 {
        self.retainer_z() + self.retainer_h() + self.thrust_float
    }
    fn wing_angle_deg(&self, index: usize) -> f64 {
        self.wing_offset_deg + 360.0 / self.wing_count.max(1) as f64 * index as f64
    }
    fn helix_azimuth_deg(&self, index: usize, t: f64) -> f64 {
        self.wing_angle_deg(index) + self.helix_deg * t
    }
    fn helix_center(&self, index: usize, t: f64) -> [f64; 2] {
        let radians = self.helix_azimuth_deg(index, t).to_radians();
        let r = self.wing_radius();
        [r * radians.cos(), r * radians.sin()]
    }
    fn post_xy(&self, index: usize) -> [f64; 2] {
        let radians = (360.0 / self.post_count.max(1) as f64 * index as f64).to_radians();
        let r = self.post_circle_r();
        [r * radians.cos(), r * radians.sin()]
    }
    fn roller_xy(&self, index: usize) -> [f64; 2] {
        let radians = (360.0 / self.roller_count.max(1) as f64 * index as f64).to_radians();
        let r = self.pcd() * 0.5;
        [r * radians.cos(), r * radians.sin()]
    }
    fn solidity(&self) -> f64 {
        (self.wing_count as f64) * self.chord_root() / (std::f64::consts::PI * self.rotor_d())
    }
    fn airfoil_ok(&self) -> bool {
        self.airfoil.to_ascii_uppercase().contains("NACA")
            && self.airfoil_t_c >= 0.18
            && self.airfoil_t_c <= 0.26
            && (self.wing_thick - self.wing_chord_root * self.airfoil_t_c).abs() < 0.2
            && self.te_min() + 1e-9 >= self.nozzle_mm * 2.0
            && self.chord_root() > self.chord_tip()
    }
    fn fits_ok(&self) -> bool {
        (self.fit_running_mm - self.nozzle_mm).abs() < 1e-9
            && self.fit_friction_mm + 1e-9 < self.fit_slip_mm
            && self.fit_slip_mm + 1e-9 < self.fit_running_mm
            && (self.clearance_mm - self.fit_running_mm).abs() < 1e-9
    }
    fn rollers_ok(&self) -> bool {
        self.roller_count >= 6
            && self.roller_d() + 1e-9 >= self.roller_min_d
            && self.pcd() > self.inner_race_d()
            && self.hub_bore() > self.inner_race_d() + 2.0 * self.roller_d()
            && self.cage_od() + 1e-9 < self.hub_bore()
            && self.axle_flange_d() + 1e-9 > self.hub_od()
    }
    fn helix_ok(&self) -> bool {
        self.helix_deg >= 45.0 && self.helix_stations >= 2
    }
    fn scale_ok(&self) -> bool {
        self.scale > 0.0
            && self.scale <= self.max_scale + 1e-9
            && self.fits_x2d_at_max()
            && self.printer.bed_mm[2] >= 260.0
    }
    fn print_flat_ok(&self) -> bool {
        let axle_h = self.axle_flange_h() + self.cage_h();
        axle_h <= self.axle_flange_d() && self.rotor_print_h() > axle_h * 3.0
    }
    fn sanity_ok(&self) -> bool {
        self.base_envelope() / self.rotor_d() <= 1.55
            && self.wing_h() + 1e-9 >= self.chord_root() * 2.5
            && self.solidity() >= 0.24
            && self.solidity() <= 0.45
    }
    fn estimated_solid_cm3(&self) -> f64 {
        let hub = std::f64::consts::PI
            * ((self.hub_od() * 0.5).powi(2) - (self.hub_bore() * 0.5).powi(2))
            * self.hub_h();
        let wings = (self.wing_count as f64)
            * 0.62
            * ((self.chord_root() + self.chord_tip()) * 0.5)
            * self.wing_thick()
            * self.wing_h();
        let base = std::f64::consts::PI * (self.base_boss_d() * 0.5).powi(2) * self.base_h()
            + (self.post_count as f64) * self.rib_w() * self.post_circle_r() * self.base_h();
        let axle = std::f64::consts::PI
            * (self.axle_flange_d() * 0.5).powi(2)
            * self.axle_flange_h()
            + self.axle_square().powi(2) * self.axle_square_h();
        let cage = std::f64::consts::PI
            * ((self.cage_od() * 0.5).powi(2) - (self.cage_id() * 0.5).powi(2))
            * self.cage_h();
        let rollers = (self.roller_count as f64)
            * std::f64::consts::PI
            * (self.roller_d() * 0.5).powi(2)
            * self.roller_h();
        let retainer = std::f64::consts::PI
            * ((self.retainer_od() * 0.5).powi(2) - (self.retainer_id() * 0.5).powi(2))
            * self.retainer_h();
        (hub + wings + base + axle + cage + rollers + retainer) / 1000.0
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

#[allow(dead_code)]
struct Built {
    base_id: u64,
    axle_id: u64,
    rotor_id: u64,
    cage_id: u64,
    roller_ids: Vec<u64>,
    retainer_id: u64,
    assembly_ok: bool,
    assembly_detail: String,
    drawing_ok: bool,
    drawing_detail: String,
    blank_ok: bool,
    blank_detail: String,
}

pub fn run(call: &mut impl FnMut(&str, Value) -> Result<Value, String>) -> Result<Report, String> {
    let spec = load_spec()?;
    if !spec.fits_ok() {
        return Ok(Report::fail(
            &spec.id,
            "fits",
            "running/slip/friction stack is not a 0.4 mm PLA set",
        ));
    }

    let mut step = 0u32;
    let mut call = |name: &str, arguments: Value| {
        step += 1;
        call(name, arguments).map_err(|error| format!("step {step} {name}: {error}"))
    };

    call("cad_new_project", json!({}))?;
    let blank_detail = require_blank_document(&mut call)?;
    call(
        "cad_set_document_name",
        json!({ "name": spec.document_name }),
    )?;

    let base_id = build_base(&mut call, &spec)?;
    let axle_id = build_axle(&mut call, &spec, &[base_id])?;
    let rotor_id = build_rotor(&mut call, &spec, &[base_id, axle_id])?;
    let mut known = vec![base_id, axle_id, rotor_id];
    let (cage_id, roller_ids) = build_cartridge(&mut call, &spec, &known)?;
    known.push(cage_id);
    known.extend(roller_ids.iter().copied());
    let retainer_id = build_retainer(&mut call, &spec, &known)?;

    let (assembly_ok, assembly_detail) = match form_assembly(
        &mut call,
        &spec,
        base_id,
        axle_id,
        rotor_id,
        cage_id,
        &roller_ids,
        retainer_id,
    ) {
        Ok(()) => (true, "5 components; hub freewheels on the fixed post".to_string()),
        Err(error) => (false, error),
    };
    let (drawing_ok, drawing_detail) = match make_assembly_drawing(&mut call, &spec) {
        Ok(()) => (true, "A3 assembly sheet with fit / scale / print / BOM notes".to_string()),
        Err(error) => (false, error),
    };

    call(
        "cad_set_focus",
        json!({ "focus": "print", "explicit": true }),
    )?;
    for (id, preset) in [
        (base_id, "bambu.pla.basic.black"),
        (axle_id, "bambu.pla.basic.jade_white"),
        (rotor_id, "bambu.pla.basic.green"),
        (cage_id, "bambu.pla.matte.dark_gray"),
        (retainer_id, "bambu.pla.basic.red"),
    ] {
        call(
            "set_body_appearance",
            json!({ "body_id": id, "preset_id": preset }),
        )?;
    }
    for id in &roller_ids {
        call(
            "set_body_appearance",
            json!({ "body_id": id, "preset_id": "bambu.pla.basic.orange" }),
        )?;
    }
    let hide_detail = hide_construction(&mut call)?;
    let preflight = call("solid_export_preflight", json!({}))?;
    let plate_exports = export_print_plates(
        &mut call,
        &spec,
        base_id,
        axle_id,
        rotor_id,
        cage_id,
        &roller_ids,
        retainer_id,
    )?;
    let scene = call("solid_scene", json!({}))?;
    let document = call("cad_document", json!({}))?;
    let assembly = call("assembly_document", json!({})).unwrap_or(json!({}));
    Ok(grade(
        &spec,
        &scene,
        &document,
        &assembly,
        &preflight,
        &plate_exports,
        Built {
            base_id,
            axle_id,
            rotor_id,
            cage_id,
            roller_ids,
            retainer_id,
            assembly_ok,
            assembly_detail,
            drawing_ok,
            drawing_detail,
            blank_ok: true,
            blank_detail: format!("{blank_detail}; {hide_detail}"),
        },
    ))
}

fn export_print_plates(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    base_id: u64,
    axle_id: u64,
    rotor_id: u64,
    cage_id: u64,
    roller_ids: &[u64],
    retainer_id: u64,
) -> Result<Vec<(String, Value)>, String> {
    let mut cartridge = vec![cage_id];
    cartridge.extend(roller_ids.iter().copied());
    let jobs = [
        ("01-base", vec![base_id]),
        ("02-axle", vec![axle_id]),
        ("03-rotor", vec![rotor_id]),
        ("04-roller-cartridge", cartridge),
        ("05-retainer", vec![retainer_id]),
    ];
    let mut plates = Vec::new();
    for (name, body_ids) in jobs {
        let exported = call(
            "solid_export_3mf",
            json!({
                "slicer_target": spec.slicer_target,
                "include_appearance": true,
                "body_ids": body_ids
            }),
        )?;
        plates.push((name.to_string(), exported));
    }
    Ok(plates)
}

fn build_base(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    begin_xy(call)?;
    add_circle(call, [0.0, 0.0], spec.base_boss_d())?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.base_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "base boss",
    )?;
    let base_id = newest_body_id(&update, &[])?;
    for index in 0..spec.post_count {
        let [px, py] = spec.post_xy(index);
        let angle = 360.0 / spec.post_count.max(1) as f64 * index as f64;
        begin_xy(call)?;
        add_oriented_rect(
            call,
            [px * 0.5, py * 0.5],
            spec.post_circle_r(),
            spec.rib_w(),
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
                    "extent": { "type": "distance", "distance": spec.base_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [base_id]
                }),
            )?,
            &format!("base rib {index}"),
        )?;
        begin_xy(call)?;
        add_circle(call, [px, py], spec.pad_d())?;
        let pad = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": pad,
                    "profile_indices": [0],
                    "operation": "join",
                    "extent": { "type": "distance", "distance": spec.base_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [base_id]
                }),
            )?,
            &format!("base pad {index}"),
        )?;
    }
    begin_xy(call)?;
    add_oriented_rect(
        call,
        [0.0, 0.0],
        spec.axle_square(),
        spec.axle_square(),
        0.0,
    )?;
    let post = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": post,
                "profile_indices": [0],
                "operation": "join",
                "extent": { "type": "distance", "distance": spec.post_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [base_id]
            }),
        )?,
        "base square post",
    )?;
    Ok(base_id)
}

fn build_axle(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.flange_z())?;
    begin_datum(call, deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.axle_flange_d())?;
    let flange = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": flange,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.axle_flange_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "axle flange",
    )?;
    let axle_id = newest_body_id(&update, known)?;
    let race_deck = offset_xy(call, spec.race_z())?;
    begin_datum(call, race_deck)?;
    add_circle(call, [0.0, 0.0], spec.inner_race_d())?;
    let race = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": race,
                "profile_indices": [0],
                "operation": "join",
                "extent": { "type": "distance", "distance": spec.cage_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [axle_id]
            }),
        )?,
        "axle inner race",
    )?;
    begin_datum(call, deck)?;
    add_oriented_rect(
        call,
        [0.0, 0.0],
        spec.hub_square(),
        spec.hub_square(),
        0.0,
    )?;
    let bore = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": bore,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.axle_flange_h() + spec.cage_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [axle_id]
            }),
        )?,
        "axle square bore",
    )?;
    Ok(axle_id)
}

fn build_rotor(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<u64, String> {
    let z0 = spec.hub_z();
    let deck = offset_xy(call, z0)?;
    begin_datum(call, deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.hub_od())?;
    add_circle(call, [0.0, 0.0], spec.hub_bore())?;
    let hub = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": hub,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.hub_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "rotor hub",
    )?;
    let rotor_id = newest_body_id(&update, known)?;

    for index in 0..spec.wing_count {
        let [px, py] = spec.helix_center(index, 0.0);
        let spar_deck = offset_xy(call, z0)?;
        begin_datum(call, spar_deck)?;
        add_oriented_rect(
            call,
            [px * 0.5, py * 0.5],
            spec.wing_radius() + spec.chord_root() * 0.5,
            spec.wall() * 3.0,
            spec.wing_angle_deg(index),
        )?;
        let spar = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": spar,
                    "profile_indices": [0],
                    "operation": "join",
                    "extent": { "type": "distance", "distance": spec.hub_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [rotor_id]
                }),
            )?,
            &format!("blade spar {index}"),
        )?;

        let stations = spec.helix_stations.max(2);
        let mut sections = Vec::new();
        for station in 0..stations {
            let t = station as f64 / (stations - 1) as f64;
            let chord = spec.chord_root() * (1.0 - t) + spec.chord_tip() * t;
            let deck = offset_xy(call, z0 + spec.hub_h() * 0.35 + spec.wing_h() * t)?;
            begin_datum(call, deck)?;
            add_airfoil(
                call,
                spec.helix_center(index, t),
                spec.helix_azimuth_deg(index, t) + 90.0,
                chord,
                spec.airfoil_t_c,
                spec.airfoil_stations,
                spec.te_min(),
            )?;
            sections.push(finish_sketch(call)?);
        }
        require_clean(
            call(
                "solid_loft",
                json!({
                    "sections": sections.iter().map(|name| json!({
                        "sketch_name": name,
                        "profile_index": 0
                    })).collect::<Vec<_>>(),
                    "ruled": false,
                    "operation": "join",
                    "target_body_ids": [rotor_id],
                    "continuity": "g1"
                }),
            )?,
            &format!("helical blade {index}"),
        )?;
    }
    Ok(rotor_id)
}

fn build_cartridge(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<(u64, Vec<u64>), String> {
    let deck = offset_xy(call, spec.race_z())?;
    begin_datum(call, deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.cage_od())?;
    add_circle(call, [0.0, 0.0], spec.cage_id())?;
    let ring = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": ring,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.cage_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "roller cage",
    )?;
    let cage_id = newest_body_id(&update, known)?;
    for index in 0..spec.roller_count {
        let [x, y] = spec.roller_xy(index);
        begin_datum(call, deck.clone())?;
        add_circle(call, [x, y], spec.cage_pocket())?;
        let pocket = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": pocket,
                    "profile_indices": [0],
                    "operation": "cut",
                    "extent": { "type": "distance", "distance": spec.cage_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [cage_id]
                }),
            )?,
            &format!("cage pocket {index}"),
        )?;
    }
    let mut roller_ids = Vec::new();
    let mut seen = known.to_vec();
    seen.push(cage_id);
    for index in 0..spec.roller_count {
        let [x, y] = spec.roller_xy(index);
        begin_datum(call, deck.clone())?;
        add_circle(call, [x, y], spec.roller_d())?;
        let roller = finish_sketch(call)?;
        let update = require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": roller,
                    "profile_indices": [0],
                    "operation": "new_body",
                    "extent": { "type": "distance", "distance": spec.roller_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": []
                }),
            )?,
            &format!("roller {index}"),
        )?;
        let id = newest_body_id(&update, &seen)?;
        seen.push(id);
        roller_ids.push(id);
    }
    Ok((cage_id, roller_ids))
}

fn build_retainer(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.retainer_z())?;
    begin_datum(call, deck)?;
    add_circle(call, [0.0, 0.0], spec.retainer_od())?;
    add_circle(call, [0.0, 0.0], spec.retainer_id())?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.retainer_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "retainer",
    )?;
    newest_body_id(&update, known)
}

fn form_assembly(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    base_id: u64,
    axle_id: u64,
    rotor_id: u64,
    cage_id: u64,
    roller_ids: &[u64],
    retainer_id: u64,
) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "assembly", "explicit": true }),
    )?;
    let _ = call("cad_set_workspace", json!({ "workspace": "assembly" }));
    let mut cartridge = vec![cage_id];
    cartridge.extend(roller_ids.iter().copied());
    let mut base_occurrence_id = None;
    for (name, body_ids) in [
        ("base", vec![base_id]),
        ("axle", vec![axle_id]),
        ("rotor", vec![rotor_id]),
        ("roller_cage", cartridge),
        ("retainer", vec![retainer_id]),
    ] {
        let created = call(
            "assembly_create_component",
            json!({
                "name": name,
                "body_ids": body_ids,
                "absorb_promoted_bodies": true
            }),
        )?;
        let component_id = created["id"]
            .as_u64()
            .or_else(|| created["component"]["id"].as_u64())
            .ok_or_else(|| format!("component {name} missing id: {created}"))?;
        let document = call("assembly_document", json!({}))?;
        let occurrence_id = authored_occurrence_id(&document, component_id)
            .ok_or_else(|| format!("component {name} has no root occurrence"))?;
        if name == "base" {
            base_occurrence_id = Some(occurrence_id);
        }
    }
    if let Some(occurrence_id) = base_occurrence_id {
        call(
            "assembly_set_occurrence_grounded",
            json!({ "occurrence_id": occurrence_id, "grounded": true }),
        )?;
    }
    let _ = call(
        "assembly_set_grounded_body",
        json!({ "body_id": base_id }),
    );
    let scene = call("solid_scene", json!({}))?;
    let axle_face = planar_face_on_axis(
        &scene,
        axle_id,
        spec.race_z() + spec.cage_h(),
        spec.inner_race_d() * 0.5 + 2.0,
    )
    .ok_or_else(|| "no on-axis axle race face for the revolute".to_string())?;
    let rotor_face = planar_face_on_axis(
        &scene,
        rotor_id,
        spec.hub_z() + spec.hub_h(),
        spec.hub_od() * 0.5 + 1.0,
    )
    .ok_or_else(|| "no on-axis hub face for the revolute".to_string())?;
    require_axis_frame(&axle_face, "axle revolute")?;
    require_axis_frame(&rotor_face, "rotor revolute")?;
    call(
        "assembly_create_joint",
        json!({
            "name": "rotor_spin",
            "kind": "revolute",
            "connector_a": axle_face,
            "connector_b": rotor_face,
            "grounded_body_id": axle_id
        }),
    )?;
    let document = call("assembly_document", json!({}))?;
    let defs = document["component_structure"]["definitions"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let occs = document["component_structure"]["occurrences"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if defs != 5 || occs != 5 {
        return Err(format!(
            "expected 5 parts / 5 occurrences, got {defs} components / {occs} occurrences"
        ));
    }
    Ok(())
}

fn make_assembly_drawing(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "drawing", "explicit": true }),
    )?;
    call(
        "cad_drawing_create_sheet",
        json!({
            "standard": "iso",
            "format": "a3",
            "orientation": "landscape",
            "title": spec.title,
            "drawing_number": "PK-VAWT-001",
            "revision": "A"
        }),
    )?;
    call("cad_drawing_auto_layout", json!({}))?;
    let notes = [
        (
            [18.0, 28.0],
            format!(
                "ASSEMBLY  scale={:.2} (1.0 = {} max)  PLA  nozzle {:.1} mm",
                spec.scale, spec.printer.name, spec.nozzle_mm
            ),
        ),
        (
            [18.0, 38.0],
            format!(
                "FITS  running +{:.2}  slip +{:.2}  friction +{:.2}  (slicer XY hole comp = 0)",
                spec.fit_running_mm, spec.fit_slip_mm, spec.fit_friction_mm
            ),
        ),
        (
            [18.0, 48.0],
            "PRINT  base/axle/cage/retainer FLAT. Rotor STANDING on hub, open drafted tips. Cartridge is PIP.".to_string(),
        ),
        (
            [18.0, 58.0],
            format!(
                "BOM  base (Y-frame + square post) · axle (inner-race puck) · rotor (hub+3×{}) · roller cage + {} PIP rollers · retainer",
                spec.airfoil, spec.roller_count
            ),
        ),
        (
            [18.0, 68.0],
            format!(
                "ROLLERS  Ø{:.1} × {:.1}  PCD {:.1}  for blade-tip moment. No metal 608.",
                spec.roller_d(),
                spec.roller_h(),
                spec.pcd()
            ),
        ),
    ];
    for (position, text) in notes {
        call(
            "cad_drawing_add_note",
            json!({
                "position": position,
                "text": text
            }),
        )?;
    }
    let drawing = call("cad_drawing_document", json!({}))?;
    let sheets = drawing["sheets"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if sheets < 1 {
        return Err("drawing sheet missing".to_string());
    }
    Ok(())
}

fn authored_occurrence_id(document: &Value, component_id: u64) -> Option<u64> {
    let occurrences = document["component_structure"]["occurrences"].as_array()?;
    let matches: Vec<&Value> = occurrences
        .iter()
        .filter(|occurrence| occurrence["component_id"].as_u64() == Some(component_id))
        .collect();
    if matches.len() == 1 {
        return matches[0]["id"].as_u64();
    }
    matches
        .iter()
        .find(|occurrence| {
            !occurrence["name"]
                .as_str()
                .is_some_and(|name| name.ends_with("_1"))
        })
        .or_else(|| matches.first())
        .and_then(|occurrence| occurrence["id"].as_u64())
}

fn require_axis_frame(connector: &Value, label: &str) -> Result<(), String> {
    let origin = xyz(&connector["frame"]["origin"])
        .ok_or_else(|| format!("{label}: connector frame missing origin"))?;
    let radial = origin[0].hypot(origin[1]);
    if radial > 4.0 {
        return Err(format!(
            "{label}: face is {radial:.1} mm off the axis — do not mate a blade spar"
        ));
    }
    Ok(())
}

fn planar_face_on_axis(scene: &Value, body_id: u64, z: f64, max_xy: f64) -> Option<Value> {
    let body = scene["bodies"]
        .as_array()?
        .iter()
        .find(|body| body["id"].as_u64() == Some(body_id))?;
    let mut best: Option<(&Value, f64)> = None;
    for face in body["faces"].as_array()? {
        let plane = face.get("plane")?;
        let origin = xyz(&plane["origin"])?;
        let normal = xyz(&plane["normal"])?;
        if normal[2].abs() < 0.85 {
            continue;
        }
        let radial = origin[0].hypot(origin[1]);
        if radial > max_xy {
            continue;
        }
        let score = (origin[2] - z).abs() + radial * 0.05;
        if best.is_none_or(|(_, current)| score < current) {
            best = Some((face, score));
        }
    }
    let (face, _) = best?;
    let plane = face.get("plane")?;
    Some(json!({
        "body_id": body_id,
        "face_id": face["id"],
        "face_key": face["key"],
        "kind": "planar_face",
        "frame": {
            "origin": plane["origin"],
            "primary_axis": plane["normal"],
            "secondary_axis": plane["u"]
        }
    }))
}

fn grade(
    spec: &Spec,
    scene: &Value,
    document: &Value,
    assembly: &Value,
    preflight: &Value,
    plate_exports: &[(String, Value)],
    built: Built,
) -> Report {
    let bodies = scene["bodies"].as_array().cloned().unwrap_or_default();
    let features = document["features"].as_array().cloned().unwrap_or_default();
    let rotor = bodies
        .iter()
        .find(|body| body["id"].as_u64() == Some(built.rotor_id));
    let plate_bytes: Vec<Vec<u8>> = plate_exports
        .iter()
        .map(|(_, exported)| decode_3mf_bytes(exported))
        .collect();
    let bytes_len: usize = plate_bytes.iter().map(Vec::len).sum();
    let component_count = assembly["component_structure"]["definitions"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let joint_count = assembly["joints"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let occurrence_count = assembly["component_structure"]["occurrences"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);

    let mut lessons = Vec::new();
    push_lesson(
        &mut lessons,
        "blank",
        built.blank_ok,
        built.blank_detail.clone(),
    );
    push_lesson(
        &mut lessons,
        "fits",
        spec.fits_ok(),
        format!(
            "running +{:.2}  slip +{:.2}  friction +{:.2}",
            spec.fit_running_mm, spec.fit_slip_mm, spec.fit_friction_mm
        ),
    );
    push_lesson(
        &mut lessons,
        "no_press",
        spec.fit_friction_mm > 0.0 && spec.fit_friction_mm < spec.nozzle_mm,
        "friction locate is a modeled gap; retain with flange/retainer".to_string(),
    );
    push_lesson(
        &mut lessons,
        "assemble",
        built.assembly_ok
            && component_count == 5
            && occurrence_count == 5
            && bodies.len() >= spec.min_bodies
            && built.roller_ids.len() == spec.roller_count,
        format!(
            "{} bodies, {} components, {} occurrences, {} joints; {}",
            bodies.len(),
            component_count,
            occurrence_count,
            joint_count,
            built.assembly_detail
        ),
    );
    push_lesson(
        &mut lessons,
        "rollers",
        spec.rollers_ok() && built.roller_ids.len() == spec.roller_count,
        format!(
            "{}× Ø{:.1} rollers on PCD {:.1}; hub bore {:.1}",
            spec.roller_count,
            spec.roller_d(),
            spec.pcd(),
            spec.hub_bore()
        ),
    );
    push_lesson(
        &mut lessons,
        "even",
        spec.wing_count == 3
            && spec.post_count == 3
            && (spec.wing_offset_deg + spec.helix_deg * 0.5 - 60.0).abs() < 1e-9,
        "3 blades at 120°, 60° helix from 30° root".to_string(),
    );

    let rotor_faces = rotor
        .and_then(|body| body["faces"].as_array().map(|faces| faces.len()))
        .unwrap_or(0);
    let rotor_box = rotor.and_then(bbox);
    let rotor_span = rotor_box
        .map(|box3| box3[1][2] - box3[0][2])
        .unwrap_or(0.0);
    push_lesson(
        &mut lessons,
        "one_piece_rotor",
        rotor_faces >= spec.min_rotor_faces
            && rotor_span > spec.wing_h() * 0.7
            && spec.chord_root() > spec.chord_tip(),
        format!("rotor faces={rotor_faces} span={rotor_span:.1} (hub+3 blades, open tips)"),
    );
    push_lesson(
        &mut lessons,
        "airfoil",
        spec.airfoil_ok() && rotor_faces >= spec.min_rotor_faces,
        format!(
            "{} t/c={:.2} TE≥{:.1}; σ={:.2}; root/tip chord {:.1}/{:.1}",
            spec.airfoil,
            spec.airfoil_t_c,
            spec.te_min(),
            spec.solidity(),
            spec.chord_root(),
            spec.chord_tip()
        ),
    );
    push_lesson(
        &mut lessons,
        "scale",
        spec.scale_ok() && spec.sanity_ok(),
        format!(
            "scale {:.2} of {}  rotor Ø{:.0} h{:.0}  bed {:.0}×{:.0}×{:.0}",
            spec.scale,
            spec.printer.name,
            spec.rotor_d(),
            spec.rotor_print_h(),
            spec.printer.bed_mm[0],
            spec.printer.bed_mm[1],
            spec.printer.bed_mm[2]
        ),
    );
    push_lesson(
        &mut lessons,
        "print_flat",
        spec.print_flat_ok(),
        format!(
            "axle puck h{:.1} on flange Ø{:.1}; rotor stands {:.0}",
            spec.axle_flange_h() + spec.cage_h(),
            spec.axle_flange_d(),
            spec.rotor_print_h()
        ),
    );
    push_lesson(
        &mut lessons,
        "helix",
        spec.helix_ok() && rotor_span > spec.wing_h() * 0.7,
        format!(
            "{}° helix, {} stations, open tips",
            spec.helix_deg, spec.helix_stations
        ),
    );
    push_lesson(
        &mut lessons,
        "drawing",
        built.drawing_ok,
        built.drawing_detail.clone(),
    );
    push_lesson(
        &mut lessons,
        "report",
        spec.estimated_filament_usd() > 0.05 && spec.airfoil_ok(),
        format!(
            "solid_cm3={:.1} mass_g={:.1} usd={:.2} {}",
            spec.estimated_solid_cm3(),
            spec.estimated_print_mass_g(),
            spec.estimated_filament_usd(),
            spec.filament.name
        ),
    );
    let plates_ok = plate_bytes.len() >= spec.min_print_plates
        && plate_bytes
            .iter()
            .all(|bytes| bytes.len() > 32 && bytes[0] == 0x50 && bytes[1] == 0x4b);
    push_lesson(
        &mut lessons,
        "export",
        features.iter().all(|feature| {
            feature
                .get("status")
                .and_then(|status| status.get("state"))
                .and_then(Value::as_str)
                != Some("error")
        }) && (preflight["ok"] == true
            || preflight["timeline_errors"]
                .as_array()
                .is_some_and(|errors| errors.is_empty()))
            && plates_ok,
        format!(
            "{} plates, {} bytes, preflight={}",
            plate_exports.len(),
            bytes_len,
            preflight["ok"]
        ),
    );

    Report {
        ok: lessons.iter().all(|lesson| lesson.pass),
        spec_id: spec.id.clone(),
        lessons,
        body_count: bodies.len(),
        byte_length: bytes_len,
    }
}

fn push_lesson(lessons: &mut Vec<LessonResult>, id: &str, pass: bool, detail: String) {
    lessons.push(LessonResult {
        id: id.to_string(),
        pass,
        detail,
    });
}

fn decode_3mf_bytes(exported: &Value) -> Vec<u8> {
    exported["bytes_base64"]
        .as_str()
        .and_then(|text| {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.decode(text).ok()
        })
        .unwrap_or_default()
}

fn naca_00_thickness(x: f64, thickness_ratio: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    5.0 * thickness_ratio
        * (0.2969 * x.sqrt() - 0.1260 * x - 0.3516 * x * x + 0.2843 * x.powi(3)
            - 0.1015 * x.powi(4))
}

fn naca_symmetric_loop(
    chord: f64,
    thickness_ratio: f64,
    stations: usize,
    te_min: f64,
) -> Vec<[f64; 2]> {
    let count = stations.max(6);
    let mut xs = Vec::with_capacity(count);
    for i in 0..count {
        let beta = std::f64::consts::PI * i as f64 / (count - 1) as f64;
        xs.push(0.5 * (1.0 - beta.cos()));
    }
    let mut upper = Vec::new();
    let mut lower = Vec::new();
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
    let world: Vec<[f64; 2]> = naca_symmetric_loop(chord, thickness_ratio, stations, te_min)
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

fn require_blank_document(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
) -> Result<String, String> {
    let scene = call("solid_scene", json!({}))?;
    let document = call("cad_document", json!({}))?;
    let bodies = scene["bodies"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let features = document["features"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if bodies != 0 {
        return Err(format!(
            "cad_new_project left {bodies} bodies / {features} features — do not continue a recovered document"
        ));
    }
    Ok(format!("blank: {bodies} bodies, {features} features"))
}

fn hide_construction(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
) -> Result<String, String> {
    let planes = call("construction_plane_definitions", json!({}))?;
    let plane_list = if let Some(items) = planes.as_array() {
        items.clone()
    } else {
        planes["planes"].as_array().cloned().unwrap_or_default()
    };
    let datum_ids: Vec<Value> = plane_list
        .iter()
        .filter_map(|plane| plane.get("datum_id").cloned())
        .collect();
    if datum_ids.is_empty() {
        return Err("helix stations created no construction planes to hide".to_string());
    }
    let document = call("cad_document", json!({}))?;
    let sketch_names: Vec<String> = document["features"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|feature| feature["kind"] == "sketch")
        .filter_map(|feature| feature["name"].as_str().map(str::to_string))
        .collect();
    let visibility = call(
        "cad_set_project_visibility",
        json!({
            "visibility": {
                "hidden_body_ids": [],
                "hidden_datum_plane_ids": datum_ids,
                "hidden_sketch_names": sketch_names
            }
        }),
    )?;
    let hidden = visibility["hidden_datum_plane_ids"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if hidden == 0 {
        return Err("cad_set_project_visibility did not hide construction planes".to_string());
    }
    Ok(format!(
        "hid {hidden} datums, {} sketches",
        sketch_names.len()
    ))
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
        assert_eq!(spec.id, "fdm-print-vawt");
        assert_eq!(spec.nozzle_mm, 0.4);
        assert!(spec.fits_ok());
        assert!(spec.rollers_ok());
        assert!(spec.airfoil_ok());
        assert!(spec.helix_ok());
        assert!(spec.scale_ok());
        assert!(spec.sanity_ok());
        assert!(spec.print_flat_ok());
        assert_eq!(spec.wing_count, 3);
        assert!(spec.chord_root() > spec.chord_tip());
        assert!(spec.estimated_filament_usd() > 0.05);
        assert!(spec.min_bodies >= 8);
        assert!(spec.min_print_plates >= 5);
        assert_eq!(spec.print_plates.len(), 5);
        assert!(spec.retired_print_plates.iter().any(|name| name == "02-shaft"));
        assert!(spec.retired_print_plates.iter().any(|name| name == "06-bushing"));
        assert!((spec.wing_offset_deg + spec.helix_deg * 0.5 - 60.0).abs() < 1e-9);
        assert!(spec.rotor_print_h() / spec.scale <= spec.usable_bed()[2] + 1e-6);
        assert!(spec.cage_od() < spec.hub_bore());
        assert!(spec.axle_flange_d() > spec.hub_od());
        assert!(spec.cage_id() > spec.inner_race_d());
        assert!(spec.post_h() < spec.axle_flange_d() * 2.5);
    }
}

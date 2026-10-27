//! Real CLEG commands -> retained GAPOWR slot -> production emitter/lowerer/GPU.
//!
//! The original corpus supplies uniform color/Z/A destinations, LightGold and
//! default Scenario lighting at a flat level-zero cell. Its temporal receiver
//! boundary is not a complete
//! CLEG battle. Here ordinary Attack/Stop produces the tested slot states; each
//! observed AI frame selects its matching original draw. A translated camera
//! aligns that real slot to the native point. The physical map's actual height
//! is retained; no claim of identical world placement or nonempty-depth scene
//! composition is made. GAPOWR_A has no Shadow, so the entire original Anim draw
//! is compared, without borrowing YAREFN output or editing emitted pieces.

use crate::app::presentation::instances::{PLANT_OWNER, SlotScene};
use crate::app::presentation::render::draw_plan_lowering::{ObjectTexture, lower_object_instances};
use crate::app::presentation::render::merge_passes::{ObjectDrawPhase, draw_native_object_pass};
use crate::assets::shp_file::ShpFile;
use crate::map::houses::{HouseColorMap, parse_house_roster};
use crate::map::lighting::{CellLightGrid, LightingConfig, build_cell_light_grid_from_heights};
use crate::render::batch::{BatchRenderer, InstanceBufferPool};
use crate::render::native_z::{ZGradient, pack_z_gradient, stored_z};
use crate::render::palette_light::PaletteLight;
use crate::render::shroud_buffer::ShroudBuffer;
use crate::render::sprite_atlas::{
    ShpPaletteContext, ShpSpriteKey, SpriteAtlas, build_sprite_atlas,
};
use crate::render::tactical_draw_plan::RenderZPolicy;
use crate::render::terrain_draw::TerrainDrawRenderer;
use crate::render::terrain_draw_gpu_tests::{Gpu, camera, clear, encoded};
use crate::rules::house_colors::HouseColorIndex;
use crate::sim::command::Command;
use serde_json::{Value, json};
use std::path::Path;

const SIZE: [u32; 2] = [160, 160];
const CLIP: [u32; 4] = [0, 0, 160, 160];
const PIXELS: usize = 160 * 160;

fn integer(value: &Value) -> i32 {
    i32::try_from(value.as_i64().unwrap()).unwrap()
}

fn plane(planes: &Value, key: &str) -> Vec<u16> {
    let mut words = Vec::with_capacity(PIXELS);
    for run in planes[key]
        .as_str()
        .expect("full original plane RLE")
        .split_ascii_whitespace()
    {
        let (count, word) = run.split_once(':').expect("count:word pair");
        let count: usize = count.parse().expect("unsigned run length");
        let word: u16 = word.parse().expect("uint16 native word");
        assert!(count > 0 && count <= PIXELS - words.len());
        words.resize(words.len() + count, word);
    }
    assert_eq!(words.len(), PIXELS, "complete original row-major plane");
    words
}

struct FormatEquipment {
    format: wgpu::TextureFormat,
    batch: BatchRenderer,
    atlas: SpriteAtlas,
    zshape: crate::render::building_zshape::BuildingZShape,
}

struct SlotGpu {
    gpu: Gpu,
    formats: Vec<FormatEquipment>,
    physical: ShpFile,
    light: CellLightGrid,
    house_colors: HouseColorMap,
    differences: Vec<String>,
    transport_differences: Vec<String>,
    compared: usize,
    opaque_controls: usize,
}

impl SlotGpu {
    fn new(fixture: &SlotScene, root: &Path, native: Option<&Value>) -> Self {
        let mut assets = crate::assets::asset_manager::AssetManager::new(
            root,
            crate::assets::asset_manager::MediaArchiveMode::STOCK_DIGITAL,
        )
        .unwrap();
        let theater = crate::map::theater::load_theater(&mut assets, "TEMPERATE").unwrap();
        let sim = fixture.scene.sim();
        let rules = &fixture.scene.runtime.resources.rules;
        let image = rules.art().resolve_anim_image_id("GAPOWR_A");
        let (filename, physical) = crate::rules::art_data::effect_shp_candidates(
            crate::rules::art_data::EffectImageKind::Animation,
            Some(rules.art()),
            "GAPOWR_A",
            &image,
            theater.extension,
            "TEMPERATE",
        )
        .into_iter()
        .find_map(|filename| {
            assets.get_ref(&filename).map(|bytes| {
                (
                    filename,
                    ShpFile::from_bytes(bytes).expect("physical selected GAPOWR_A SHP"),
                )
            })
        })
        .expect("registered physical active animation image");
        let config = rules.art().anim_runtime_config("GAPOWR_A").unwrap();
        assert!(!config.shadow && config.should_use_cell_drawer && !config.use_normal_light);
        assert_eq!(
            config.raw_shp_frame_count,
            Some(physical.frames.len() as i32)
        );
        for case in native
            .into_iter()
            .flat_map(|value| value["cases"].as_array().unwrap())
        {
            let reader = &case["anim_type_reader"];
            assert_eq!(reader["shadow"], false);
            assert_eq!(reader["flat"], false);
            assert_eq!(reader["should_use_cell_drawer"], true);
            assert_eq!(reader["use_normal_light"], false);
            assert_eq!(json!(config.start), reader["start"]);
            assert_eq!(json!(config.loop_end), reader["loop_end"]);
            assert_eq!(json!(physical.frames.len()), reader["raw_shp_frame_count"]);
            assert!(
                reader["bound_physical_names"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|name| name.as_str().unwrap().eq_ignore_ascii_case(&filename))
            );
        }
        let mut house_colors: HouseColorMap = parse_house_roster(
            &fixture.scene.map.ini,
            rules.color_schemes.as_slice(),
            Some(rules),
        )
        .color_map();
        // The ordinary launch house constructor selects color zero (LightGold).
        house_colors.insert(PLANT_OWNER.to_owned(), HouseColorIndex(0));
        let gpu = Gpu::new();
        let formats = [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ]
        .into_iter()
        .map(|format| {
            let batch = BatchRenderer::new_with_device(&gpu.device, &gpu.queue, format);
            let zshape = crate::render::building_zshape::BuildingZShape::load_on_device(
                &gpu.device, &gpu.queue, &batch, &assets,
            ).expect("building body probe requires the real BUILDNGZ.SHA, not the neutral one-texel fallback");
            let atlas = build_sprite_atlas(
                &gpu.device,
                &gpu.queue,
                &batch,
                sim.entities(),
                &assets,
                &theater.unit_palette,
                theater.extension,
                "TEMPERATE",
                Some(rules),
                Some(rules.art()),
                &house_colors,
                &[],
                &Default::default(),
                &Default::default(),
                &Default::default(),
                None,
                None,
                Some(sim.interner()),
            )
            .expect("production physical atlas including the real building slot");
            FormatEquipment {
                format,
                batch,
                atlas,
                zshape,
            }
        })
        .collect();
        // Explicit native component input, separate from XMP03T4's physical
        // geometry/lighting: original Scenario defaults (Ambient1000, Ground50,
        // Level8) relight a level-zero cell. Use the real relight owner rather
        // than assigning a measured output brightness. Actor/map height stays
        // untouched, and camera translation accounts for its projection.
        let plant = sim.entities().get(fixture.plant).unwrap();
        let cell = (plant.position.rx(), plant.position.ry());
        let light = build_cell_light_grid_from_heights([(cell, 0)], &LightingConfig::default());
        Self {
            gpu,
            formats,
            physical,
            light,
            house_colors,
            differences: Vec::new(),
            transport_differences: Vec::new(),
            compared: 0,
            opaque_controls: 0,
        }
    }

    /// Presence/absence probe over current physical assets, not a new native
    /// pixel golden. Both body and slots enter their production emitters.
    fn check_cloak_visibility(
        &mut self,
        fixture: &SlotScene,
        slot: u64,
        visible: bool,
        label: &str,
    ) {
        use crate::app::presentation::instances::{ShpInputs, build_shp_instances_from_inputs};
        use crate::app::presentation::render::draw_plan_lowering::NativeDisplayOrder;
        let sim = fixture.scene.sim();
        let coord = sim.anim_absolute_coord(slot).unwrap();
        let (rx, ry, sx, sy, _) = coord.to_cell_sub_z();
        let projected = crate::util::lepton::lepton_to_screen_exact_z(rx, ry, sx, sy, coord.z);
        let view_camera = [projected.0 - 80.0, projected.1 - 80.0];
        let viewer = sim
            .session()
            .current_house()
            .map(|id| sim.interner().resolve(id));
        let before = (sim.state_hash(), sim.rng_state());
        for FormatEquipment {
            format,
            batch,
            atlas,
            zshape,
        } in &mut self.formats
        {
            let mut view = camera(SIZE);
            view.camera_pos = view_camera;
            batch.write_camera(&self.gpu.queue, view);
            let fog = ShroudBuffer::fixture(
                &self.gpu.device,
                &self.gpu.queue,
                SIZE,
                view_camera,
                &vec![0x7F; PIXELS],
            );
            batch.bind_shroud(&self.gpu.device, Some(&fog));
            let mut parents = Vec::new();
            build_shp_instances_from_inputs(
                &ShpInputs {
                    runtime: &fixture.scene.runtime,
                    atlas: Some(atlas),
                    overlay_atlas: None,
                    unit_atlas: None,
                    barrel_image_pitches: &Default::default(),
                    interpolation:
                        &crate::app::presentation::interpolation::RenderInterpolation::new(false),
                    camera: view_camera,
                    render_size: SIZE,
                    zoom: 1.0,
                    axis: crate::render::batch::DepthAxis::NONE,
                    local_owner: viewer,
                    ignore_visibility: true,
                    lighting: &self.light,
                    house_colors: &self.house_colors,
                    building_zshape: true,
                },
                &mut parents,
                &NativeDisplayOrder::from_display(sim.display_layers()),
            );
            parents.retain(|parent| parent.parent.id == fixture.plant);
            let body_count = parents.len();
            let slots = fixture.emit_slot(
                atlas,
                &self.light,
                [
                    view_camera[0],
                    view_camera[1],
                    SIZE[0] as f32,
                    SIZE[1] as f32,
                ],
                &|_| 0,
            );
            assert_eq!(
                body_count > 0,
                visible,
                "{label}: real building body admission"
            );
            assert_eq!(
                !slots.is_empty(),
                visible,
                "{label}: real attached slot admission"
            );
            for (group, parents) in [("body", parents), ("slots", slots)] {
                let layers = lower_object_instances(parents);
                let color = self.gpu.target(SIZE, *format);
                let background = encoded(0x8410, *format);
                self.gpu.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &color,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &background.repeat(PIXELS),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(SIZE[0] * 4),
                        rows_per_image: Some(SIZE[1]),
                    },
                    color.size(),
                );
                let cv = color.create_view(&Default::default());
                let depth = self.gpu.target(SIZE, wgpu::TextureFormat::Depth32Float);
                let dv = depth.create_view(&Default::default());
                let mut terrain =
                    TerrainDrawRenderer::new(&self.gpu.device, &self.gpu.queue, *format, batch);
                terrain.prepare(&self.gpu.device, &color, &dv, batch.camera_uniform());
                let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
                clear(&mut encoder, &cv, &dv, u16::MAX, wgpu::LoadOp::Load);
                let mut pool = InstanceBufferPool::new();
                for (index, layer) in layers
                    .iter()
                    .enumerate()
                    .filter(|(_, layer)| !layer.instances.is_empty())
                {
                    let key = ["cloak0", "cloak1", "cloak2", "cloak3", "cloak4"][index];
                    pool.upload_on_device(&self.gpu.device, &self.gpu.queue, key, &layer.instances);
                    // Buildings6D9920 draws the base before Display6D916C;
                    // slots use Display, but the body is not in those runs.
                    for phase in [ObjectDrawPhase::BuildingBase, ObjectDrawPhase::Display] {
                        draw_native_object_pass(
                            &mut encoder,
                            phase,
                            &cv,
                            &dv,
                            &mut terrain,
                            CLIP,
                            batch,
                            pool.get(key),
                            layer,
                            None,
                            None,
                            None,
                            Some(atlas),
                            None,
                            &zshape.bind_group,
                            None,
                            None,
                        );
                    }
                }
                let read = self.gpu.read(&mut encoder, &color);
                let output = self.gpu.finish(encoder, &[read], SIZE);
                let changed = output[0]
                    .chunks_exact(4)
                    .filter(|pixel| *pixel != background.as_slice())
                    .count();
                if visible && changed == 0 {
                    eprintln!(
                        "GPU input camera={view_camera:?} layers={:?}",
                        layers
                            .iter()
                            .map(|layer| (
                                layer.runs.len(),
                                layer.building_base_runs.len(),
                                &layer.instances,
                            ))
                            .collect::<Vec<_>>()
                    );
                }
                eprintln!("cloak GPU {label}/{group}/{format:?}: changed pixels={changed}");
                assert_eq!(
                    changed > 0,
                    visible,
                    "{label}/{group}/{format:?}: actual GPU changed {changed} pixels"
                );
            }
        }
        assert_eq!(
            (sim.state_hash(), sim.rng_state()),
            before,
            "read-only local drawing must not mutate simulation"
        );
    }

    fn compare_phase(
        &mut self,
        fixture: &SlotScene,
        slot: u64,
        phase: &str,
        erased: bool,
        native: &Value,
    ) {
        fixture.assert_slot(slot, erased);
        let sim = fixture.scene.sim();
        let anim = sim.anim(slot).unwrap();
        let frame = anim.runtime().current_frame;
        assert!(
            (0..8).contains(&frame),
            "actual command-produced frame has an original sample"
        );
        let row_name = format!("existing_stage{frame}_{phase}");
        let coord = sim.anim_absolute_coord(slot).unwrap();
        let (rx, ry, sx, sy, _) = coord.to_cell_sub_z();
        let projected = crate::util::lepton::lepton_to_screen_exact_z(rx, ry, sx, sy, coord.z);
        let actual_lift = crate::util::native_x87::adjust_for_z_standard(coord.z);
        let before = (sim.state_hash(), sim.rng_state());
        for case in native["cases"].as_array().unwrap() {
            let name = case["input"]["name"].as_str().unwrap();
            let row = case["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["name"] == row_name)
                .expect("matching actually reached native AI frame");
            let state = &row["state_before"]["anim"];
            assert_eq!(state["runtime"]["current_frame"], frame);
            assert_eq!(state["temporal_paused"], erased);
            assert_eq!(state["forced_translucency"], erased);
            assert_eq!(row["state_before"]["building"]["warped"], erased);
            assert_eq!(row["state_before"]["building"]["online"], !erased);
            assert_eq!(row["size"], json!(SIZE));
            let calls = row["calls"].as_array().unwrap();
            assert_eq!(
                calls.len(),
                1,
                "physical Shadow=no gives a complete one-call draw"
            );
            let call = &calls[0];
            assert_eq!(call["frame"], frame);
            assert_eq!(call["clip"], json!(CLIP));
            assert_eq!(call["color"], 0, "declared neutral caller color word");
            assert_eq!(call["gradient"], 2);
            assert_eq!(call["has_zshape"], false);
            assert_eq!(row["a_unchanged"], true);
            assert_eq!(row["z_unchanged"], true);
            assert!(row["rng_entries"].as_array().unwrap().is_empty());
            for selector in call["selectors"].as_array().unwrap() {
                assert_eq!(selector["half_mask"], 0x7BEF);
                assert_eq!(selector["quarter_mask"], 0x39E7);
                assert!(
                    selector["converter_identity"]["matches"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|name| name == "constructed_LightGold_local_index_1"),
                    "original draw selects the actual LightGold converter"
                );
            }
            let point: [f32; 2] = std::array::from_fn(|i| integer(&call["point"][i]) as f32);
            let view_camera = [projected.0 - point[0], projected.1 - point[1]];
            let native_z = integer(&state["stored_xyz"][2]);
            let native_lift = crate::util::native_x87::adjust_for_z_standard(native_z);
            let raw = &self.physical.frames[frame as usize];
            assert_eq!(
                json!([self.physical.width, self.physical.height]),
                call["source_frame"]["canvas"]
            );
            assert_eq!(
                json!([raw.frame_x, raw.frame_y, raw.frame_width, raw.frame_height]),
                call["source_frame"]["crop"]
            );
            let background = u16::try_from(case["input"]["background"].as_u64().unwrap()).unwrap();
            let old_z = u16::try_from(case["input"]["initial_z"].as_u64().unwrap()).unwrap();
            let a_value = u8::try_from(case["input"]["a_value"].as_u64().unwrap()).unwrap();
            assert_eq!(old_z, u16::MAX, "declared empty-depth comparison boundary");
            assert_eq!(
                plane(&call["before_planes"], "color_words_rle"),
                vec![background; PIXELS]
            );
            assert_eq!(
                plane(&call["before_planes"], "z_words_rle"),
                vec![old_z; PIXELS]
            );
            assert_eq!(
                plane(&call["before_planes"], "a_words_rle"),
                vec![u16::from(a_value); PIXELS]
            );
            let expected = plane(&call["after_planes"], "color_words_rle");
            let expected_z = plane(&call["after_planes"], "z_words_rle");
            let expected_a = plane(&call["after_planes"], "a_words_rle");
            for equipment in &mut self.formats {
                let FormatEquipment {
                    format,
                    batch,
                    atlas,
                    ..
                } = equipment;
                let mut view = camera(SIZE);
                view.camera_pos = view_camera;
                batch.write_camera(&self.gpu.queue, view);
                let fog = ShroudBuffer::fixture(
                    &self.gpu.device,
                    &self.gpu.queue,
                    SIZE,
                    view_camera,
                    &vec![a_value; PIXELS],
                );
                batch.bind_shroud(&self.gpu.device, Some(&fog));
                let key = ShpSpriteKey {
                    palette_context: ShpPaletteContext::AnimScheme,
                    type_id: "GAPOWR_A".into(),
                    facing: 0,
                    frame: frame as u16,
                    house_color: HouseColorIndex(0),
                };
                let entry = atlas.get(&key).expect("real selected-scheme frame");
                assert_ne!(entry.source_palette, 0);
                let parents = fixture.emit_slot(
                    atlas,
                    &self.light,
                    [
                        view_camera[0],
                        view_camera[1],
                        SIZE[0] as f32,
                        SIZE[1] as f32,
                    ],
                    &|_| 0,
                );
                assert_eq!(
                    parents.len(),
                    1,
                    "only the actual GAPOWR active slot is emitted"
                );
                assert_eq!(parents[0].parent.id, slot);
                assert_eq!(parents[0].pieces.len(), 1);
                let layer_index = parents[0].parent.layer.0 as usize;
                let piece = &parents[0].pieces[0];
                assert_eq!(piece.target, ObjectTexture::ShpPage(entry.page as usize));
                assert_eq!(piece.render_z, RenderZPolicy::ReadOnly);
                let sprite = &piece.instance;
                assert_eq!(sprite.source_palette, entry.source_palette);
                assert_eq!(sprite.uv_origin, entry.uv_origin);
                assert_eq!(sprite.uv_size, entry.uv_size);
                assert_eq!(
                    sprite.size,
                    [f32::from(raw.frame_width), f32::from(raw.frame_height)]
                );
                assert_eq!(
                    [
                        sprite.position[0] - view_camera[0],
                        sprite.position[1] - view_camera[1]
                    ],
                    [
                        point[0] + f32::from(raw.frame_x) - f32::from(self.physical.width / 2),
                        point[1] + f32::from(raw.frame_y) - f32::from(self.physical.height / 2)
                    ]
                );
                assert_eq!(
                    sprite.z_adjust,
                    (integer(&call["z_adjust"]) + native_lift - actual_lift) as f32,
                    "same native Z term with the actual map's height retained"
                );
                assert_eq!(
                    sprite.z_gradient,
                    pack_z_gradient(ZGradient::Vertical, false)
                );
                // The default-profile CellLightGrid is an explicit caller input, but
                // final brightness belongs to production body_extra_light ->
                // body_palette_light -> building_anim_light. Compare its
                // untouched output to the measured original call; never fit
                // the grid or emitted piece to that result. Defer a mismatch
                // until after pixel replay so it remains useful evidence.
                let expected_light =
                    PaletteLight::color_scheme([1000; 3], integer(&call["intensity"]));
                if sprite.palette_light != expected_light {
                    self.transport_differences.push(format!(
                        "{name}/{row_name}/{format:?}: production palette {:?}, measured original {:?}",
                        sprite.palette_light, expected_light
                    ));
                }
                let selector = call["flags"].as_u64().unwrap() as u32 & 6;
                // Measure every pixel before asserting the known transport gap.
                if sprite.draw_state.native_selector_bits() != selector || sprite.alpha != 1. {
                    self.transport_differences.push(format!(
                        "{name}/{row_name}/{format:?}: selector{} alpha{}, expected {selector}/1",
                        sprite.draw_state.native_selector_bits(),
                        sprite.alpha
                    ));
                }
                let layers = lower_object_instances(parents);
                let layer = &layers[layer_index];
                assert_eq!(layer.owners, [slot]);
                assert_eq!(layer.instances.len(), 1);
                assert_eq!(layer.runs.len(), 1);
                let expected_packed = (selector != 0).then_some(slot);
                if layer.runs[0].packed_parent != expected_packed {
                    self.transport_differences.push(format!(
                        "{name}/{row_name}/{format:?}: packed {:?}, expected {expected_packed:?}",
                        layer.runs[0].packed_parent
                    ));
                }
                let mut pool = InstanceBufferPool::new();
                pool.upload_on_device(&self.gpu.device, &self.gpu.queue, "slot", &layer.instances);
                let color = self.gpu.target(SIZE, *format);
                self.gpu.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &color,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &encoded(background, *format).repeat(PIXELS),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(SIZE[0] * 4),
                        rows_per_image: Some(SIZE[1]),
                    },
                    color.size(),
                );
                let cv = color.create_view(&Default::default());
                let depth = self.gpu.target(SIZE, wgpu::TextureFormat::Depth32Float);
                let dv = depth.create_view(&Default::default());
                let mut terrain =
                    TerrainDrawRenderer::new(&self.gpu.device, &self.gpu.queue, *format, batch);
                terrain.prepare(&self.gpu.device, &color, &dv, batch.camera_uniform());
                let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
                clear(&mut encoder, &cv, &dv, old_z, wgpu::LoadOp::Load);
                draw_native_object_pass(
                    &mut encoder,
                    ObjectDrawPhase::Display,
                    &cv,
                    &dv,
                    &mut terrain,
                    CLIP,
                    batch,
                    pool.get("slot"),
                    layer,
                    None,
                    None,
                    None,
                    Some(atlas),
                    None,
                    batch.default_zshape_bind_group(),
                    None,
                    None,
                );
                let reads = [
                    self.gpu.read(&mut encoder, &color),
                    self.gpu.read(&mut encoder, &depth),
                ];
                let output = self.gpu.finish(encoder, &reads, SIZE);
                let (mut color_errors, mut z_errors, mut first_error) = (0, 0, None);
                for (at, &word) in expected.iter().enumerate() {
                    let actual: [u8; 4] = output[0][at * 4..at * 4 + 4].try_into().unwrap();
                    let wanted = encoded(word, *format);
                    if actual != wanted {
                        color_errors += 1;
                        first_error.get_or_insert((at % 160, at / 160, actual, wanted));
                    }
                    z_errors += usize::from(
                        stored_z(f32::from_le_bytes(
                            output[1][at * 4..at * 4 + 4].try_into().unwrap(),
                        )) != expected_z[at],
                    );
                }
                let a_errors = self
                    .gpu
                    .read_float_texels(fog.gpu_source().0, [0, 0], SIZE)
                    .iter()
                    .zip(&expected_a)
                    .filter(|(actual, wanted)| (actual[0] * 255.).round() as u16 != **wanted)
                    .count();
                eprintln!(
                    "GAPOWR_A {name}/{row_name}/{format:?}: color {color_errors}/{PIXELS}, Z {z_errors}, A {a_errors}, first {first_error:?}"
                );
                if color_errors != 0 || z_errors != 0 || a_errors != 0 {
                    self.differences.push(format!("{name}/{row_name}/{format:?}: color={color_errors} Z={z_errors} A={a_errors}"));
                } else if !erased {
                    self.opaque_controls += 1;
                }
                self.compared += 1;
                assert_eq!(
                    (sim.state_hash(), sim.rng_state()),
                    before,
                    "emission/lowering/replay leaves simulation and all RNG unchanged"
                );
            }
        }
    }
}

#[cfg(feature = "npatch")]
#[test]
#[ignore = "requires AYMOD assets and GPU; actual cloak crate building body/slots for owner and detected/undetected viewer"]
fn aymod_cloak_crate_building_visibility_reaches_actual_gpu_output() {
    let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
    let mut fixture = SlotScene::load_registered_types(&root);
    let slot = fixture.settled_playing_slot();
    let mut gpu = SlotGpu::new(&fixture, &root, None);
    gpu.check_cloak_visibility(&fixture, slot, true, "uncloaked control");
    let tank = {
        let runtime = &mut fixture.scene.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let at = sim.entities().get(fixture.plant).unwrap().position;
        let rules = &resources.rules;
        assert!(
            !rules.object("GAPOWR").unwrap().cloakable && !rules.object("MTNK").unwrap().cloakable
        );
        let candidates = (-3_i32..=3)
            .flat_map(|dy| (-3_i32..=3).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| (i32::from(at.rx()) + dx, i32::from(at.ry()) + dy))
            .filter_map(|(x, y)| u16::try_from(x).ok().zip(u16::try_from(y).ok()))
            .filter(|&(x, y)| {
                crate::sim::cell_rect::cell_is_in_playfield_height_aware(
                    (i32::from(x), i32::from(y)),
                    sim.playfield_bounds(),
                    sim.resolved_terrain(),
                )
            })
            .collect::<Vec<_>>();
        let tank = candidates
            .iter()
            .find_map(|&(x, y)| {
                sim.spawn_object_with_overlay_registry(
                    "MTNK",
                    PLANT_OWNER,
                    x,
                    y,
                    0,
                    rules,
                    &resources.overlay_registry,
                )
            })
            .unwrap();
        let center = crate::sim::movement::ground_pose::position_world_coord(&at);
        let cell = candidates
            .into_iter()
            .find(|&(x, y)| {
                crate::util::native_x87::distance_3d_leptons(
                    [i32::from(x) * 256 + 128, i32::from(y) * 256 + 128, center.z],
                    [center.x, center.y, center.z],
                ) < rules.crate_rules.radius
                    && sim
                        .resolved_terrain()
                        .unwrap()
                        .cell(x, y)
                        .is_some_and(|cell| cell.level == at.z())
                    && sim.path_grid().unwrap().is_walkable_on_layer(
                        x,
                        y,
                        crate::sim::movement::locomotor::MovementLayer::Ground,
                    )
                    && sim.occupancy().get(x, y).is_none_or(|cell| {
                        cell.is_empty_on(crate::sim::movement::locomotor::MovementLayer::Ground)
                    })
                    && sim.overlay_grid().unwrap().cell(x, y).overlay_id.is_none()
            })
            .unwrap();
        assert!(crate::sim::crates::place_crate_for_test(
            sim,
            rules,
            &resources.overlay_registry,
            (cell.0 as i16, cell.1 as i16),
            Some(3)
        ));
        assert!(runtime.apply_command_for_test(
            PLANT_OWNER,
            &Command::Move {
                entity_id: tank,
                target_rx: cell.0,
                target_ry: cell.1,
            }
        ));
        tank
    };
    fixture.wait_for(600, "real Drive pickup cloaks the plant", |fixture| {
        [fixture.plant, tank].into_iter().all(|id| {
            fixture
                .scene
                .sim()
                .entities()
                .get(id)
                .unwrap()
                .cloak
                .as_ref()
                .is_some_and(|cloak| cloak.is_fully_cloaked())
        })
    });
    assert!(
        fixture
            .scene
            .sim()
            .entities()
            .get(tank)
            .unwrap()
            .cloak
            .as_ref()
            .unwrap()
            .is_fully_cloaked()
    );
    let owner = fixture
        .scene
        .sim()
        .entities()
        .get(fixture.plant)
        .unwrap()
        .owner();
    let viewer = fixture.scene.sim().interner().get("Americans").unwrap();
    assert_ne!(owner, viewer);
    let set_viewer = |fixture: &mut SlotScene, viewer| {
        let sim = &mut fixture.scene.runtime.simulation;
        sim.session_mut_for_test()
            .set_current_house_for_test(Some(viewer));
        sim.entities_mut_for_test()
            .get_mut(fixture.plant)
            .unwrap()
            .discovery
            .set_owned_by_current_house_for_test(viewer == owner);
    };
    set_viewer(&mut fixture, owner);
    gpu.check_cloak_visibility(&fixture, slot, true, "owner");
    set_viewer(&mut fixture, viewer);
    gpu.check_cloak_visibility(&fixture, slot, false, "unsensed enemy");
    {
        let runtime = &mut fixture.scene.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let at = sim.entities().get(fixture.plant).unwrap().position;
        let (w, h) = resources
            .rules
            .object("GAPOWR")
            .unwrap()
            .foundation
            .dimensions();
        let viewer_name = sim.interner().resolve(viewer).to_owned();
        let detector = (-1_i32..=1)
            .find_map(|dy| {
                let y = i32::from(at.ry()) + i32::from(h / 2) + dy;
                sim.spawn_object_with_overlay_registry(
                    "DOG",
                    &viewer_name,
                    at.rx() + w + 1,
                    u16::try_from(y).ok()?,
                    0,
                    &resources.rules,
                    &resources.overlay_registry,
                )
            })
            .expect("a normal detector outside the building's contact rectangle");
        assert!(sim.entities().get(detector).unwrap().is_object_alive());
        let center = crate::sim::movement::ground_pose::object_get_coords(
            sim.entities().get(fixture.plant).unwrap(),
            sim.resolved_terrain(),
        );
        assert!(sim.fog().has_sensor_for_house(
            viewer,
            crate::util::lepton::lepton_to_cell_packed(center.x) as u16,
            crate::util::lepton::lepton_to_cell_packed(center.y) as u16
        ));
    }
    gpu.check_cloak_visibility(&fixture, slot, true, "detected enemy");
}

#[test]
#[ignore = "requires retail XMP03T4 assets and GPU; ordinary CLEG Attack/Stop slot states against full original GAPOWR_A planes"]
fn retail_building_slot_matches_original_temporal_translucency() {
    let native = crate::test_fixture::json("tools/spatial_oracle/building_anim_translucency.json");
    assert_eq!(native["schema"], 1);
    assert_eq!(native["cases"].as_array().unwrap().len(), 3);
    let Some((root, _)) = crate::rules::retail_ini_fixture::retail_assets() else {
        return;
    };
    let mut fixture = SlotScene::load(&root);
    let slot = fixture.settled_playing_slot();
    let mut gpu = SlotGpu::new(&fixture, &root, Some(&native));
    gpu.compare_phase(&fixture, slot, "baseline", false, &native);
    let attacker = fixture.begin_attack();
    fixture.wait_for(240, "ordinary temporal shot links the plant", |fixture| {
        fixture
            .scene
            .sim()
            .entities()
            .get(attacker)
            .unwrap()
            .temporal
            .warp_target()
            == Some(fixture.plant)
    });
    fixture.assert_slot(slot, true);
    let frozen = fixture
        .scene
        .sim()
        .anim(slot)
        .unwrap()
        .runtime()
        .current_frame;
    for _ in 0..12 {
        fixture.step(None);
        fixture.assert_slot(slot, true);
        assert_eq!(
            fixture
                .scene
                .sim()
                .anim(slot)
                .unwrap()
                .runtime()
                .current_frame,
            frozen
        );
    }
    gpu.compare_phase(&fixture, slot, "erased_after_ai", true, &native);
    fixture.step(Some(Command::Stop {
        entity_id: attacker,
    }));
    fixture.wait_for(
        240,
        "ordinary retained Attack dispatch releases the beam",
        |fixture| {
            fixture
                .scene
                .sim()
                .entities()
                .get(attacker)
                .unwrap()
                .temporal
                .warp_target()
                .is_none()
        },
    );
    fixture.assert_slot(slot, false);
    gpu.compare_phase(&fixture, slot, "released_before_ai", false, &native);
    // The original pixel boundary above ends at LetGo, before the next
    // Building visit recreates this PoweredSpecial slot. Observe that real
    // restore edge and its new Anim AI separately from the retained draw.
    fixture.resume_after_power_restoration(slot);
    assert_eq!(
        gpu.compared, 18,
        "three command states, three underlays, both formats"
    );
    assert_eq!(
        gpu.opaque_controls, 12,
        "unchanged baseline/released controls: {:?}",
        gpu.differences
    );
    assert!(
        gpu.differences.is_empty() && gpu.transport_differences.is_empty(),
        "original plane differences: {:#?}\ntransport differences: {:#?}",
        gpu.differences,
        gpu.transport_differences
    );
}

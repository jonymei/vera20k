//! Actual AY kill reward -> ordinary Anim Display -> offscreen sprite replay.

use super::draw_plan_lowering::{NativeDisplayOrder, lower_object_instances};
use super::merge_passes::draw_native_object_pass;
use crate::app::presentation::detail::DetailState;
use crate::app::presentation::instances::{DisplayEffectInputs, build_display_effect_instances};
use crate::assets::asset_manager::{AssetManager, MediaArchiveMode};
use crate::assets::pal_file::Palette;
use crate::headless_scenario::battle_test_evidence::visit_aymod_tanya_bounty_after_load;
use crate::map::lighting::CellLightGrid;
use crate::render::batch::{BatchRenderer, DepthAxis, InstanceBufferPool};
use crate::render::sprite_atlas::{build_sprite_atlas, collect_anim_remap_base_keys};
use crate::render::terrain_draw::TerrainDrawRenderer;
use crate::render::terrain_draw_gpu_tests::{Gpu, camera, clear};
use std::cell::RefCell;

#[test]
#[ignore = "requires AYMOD and GPU; actual bounty animation through Display, restore and retirement"]
fn aymod_bounty_marker_reaches_gpu_and_retires_after_load() {
    let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
    let mut assets = AssetManager::new(&root, MediaArchiveMode::STOCK_DIGITAL).unwrap();
    crate::map::theater::load_theater(&mut assets, "TEMPERATE").unwrap();
    let palette = Palette::from_bytes(assets.get_ref("UNITTEM.PAL").unwrap()).unwrap();
    let gpu = Gpu::new();
    let size = [256, 256];
    let mut live_pixels = Vec::new();
    let mut view_camera = [0.0; 2];
    let mut visits = Vec::new();
    visit_aymod_tanya_bounty_after_load(|label, scene, id| {
        visits.push(label.to_string());
        let sim = scene.sim();
        let before = (sim.state_hash(), sim.rng_state());
        let rules = &scene.runtime.resources.rules;
        if let Some(coord) = sim.anim_absolute_coord(id) {
            let (rx, ry, sx, sy, _) = coord.to_cell_sub_z();
            let (x, y) = crate::util::lepton::lepton_to_screen_exact_z(rx, ry, sx, sy, coord.z);
            view_camera = [x - 128.0, y - 128.0];
        } else {
            assert_eq!(label, "retired");
        }
        for format in [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ] {
            let batch = BatchRenderer::new_with_device(&gpu.device, &gpu.queue, format);
            let house_colors = Default::default();
            let remaps = collect_anim_remap_base_keys(sim, &house_colors);
            let atlas = build_sprite_atlas(
                &gpu.device,
                &gpu.queue,
                &batch,
                sim.entities(),
                &assets,
                &palette,
                "tem",
                "TEMPERATE",
                Some(rules),
                Some(rules.art()),
                &house_colors,
                &[],
                &remaps,
                &Default::default(),
                &Default::default(),
                None,
                None,
                Some(sim.interner()),
            )
            .expect("physical AY animation atlas");
            let mut detail = DetailState::new();
            for _ in 0..60 {
                detail.record_logic_visit();
            }
            detail.throttle_tail(0);
            let detail = RefCell::new(detail);
            let lighting = CellLightGrid::new();
            let hidden = Default::default();
            let input = DisplayEffectInputs {
                interpolation: None,
                sim,
                rules,
                camera: view_camera,
                render_size: size,
                tactical: [0, 0, size[0], size[1]],
                zoom: 1.0,
                axis: DepthAxis::NONE,
                atlas: Some(&atlas),
                detail: &detail,
                detail_level: 2,
                lighting: &lighting,
                retained_anim_intensities: None,
                house_colors: &house_colors,
                hidden_super_anims: &hidden,
                show_hidden: false,
                local_owner: None,
                observe_particles: false,
            };
            let order = NativeDisplayOrder::from_display(sim.display_layers());
            let mut parents = Vec::new();
            build_display_effect_instances(&input, &mut parents, &order, Default::default());
            // Isolate the actual reward's Display parent, not a synthetic sprite.
            parents.retain(|parent| parent.parent.id == id);
            assert_eq!(parents.len(), usize::from(label != "retired"));
            let layers = lower_object_instances(parents);
            let mut camera_view = camera(size);
            camera_view.camera_pos = view_camera;
            batch.write_camera(&gpu.queue, camera_view);
            let color = gpu.target(size, format);
            let cv = color.create_view(&Default::default());
            let depth = gpu.target(size, wgpu::TextureFormat::Depth32Float);
            let dv = depth.create_view(&Default::default());
            let mut terrain = TerrainDrawRenderer::new(&gpu.device, &gpu.queue, format, &batch);
            terrain.prepare(&gpu.device, &color, &dv, batch.camera_uniform());
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            clear(
                &mut encoder,
                &cv,
                &dv,
                u16::MAX,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            );
            let mut pools = Vec::new();
            for layer in &layers {
                let mut pool = InstanceBufferPool::new();
                pool.upload_on_device(&gpu.device, &gpu.queue, "bounty", &layer.instances);
                draw_native_object_pass(
                    &mut encoder,
                    crate::app::render::merge_passes::ObjectDrawPhase::Display,
                    &cv,
                    &dv,
                    &mut terrain,
                    [0, 0, size[0], size[1]],
                    &batch,
                    pool.get("bounty"),
                    layer,
                    None,
                    None,
                    None,
                    Some(&atlas),
                    None,
                    batch.default_zshape_bind_group(),
                    None,
                    None,
                );
                pools.push(pool);
            }
            let reads = [gpu.read(&mut encoder, &color)];
            let pixels = gpu.finish(encoder, &reads, size).remove(0);
            let visible = pixels.chunks_exact(4).filter(|pixel| pixel[3] != 0).count();
            if label == "retired" {
                assert_eq!(
                    visible, 0,
                    "retirement removes the reward from actual output"
                );
            } else {
                assert!(
                    visible > 20,
                    "actual {label} MONEY must paint visible pixels: {visible}"
                );
                if label == "live" {
                    live_pixels.push(pixels);
                } else {
                    assert_eq!(
                        pixels,
                        live_pixels[usize::from(format == wgpu::TextureFormat::Rgba8UnormSrgb)],
                        "restore preserves rendered reward"
                    );
                }
            }
        }
        assert_eq!((sim.state_hash(), sim.rng_state()), before);
    });
    assert_eq!(visits, ["live", "restored", "retired"]);
}

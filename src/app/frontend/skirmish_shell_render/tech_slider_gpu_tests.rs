//! The real option emitters and retail font/chrome, through the production
//! batch pipeline. No window, replacement shader or handwritten glyphs.

use super::{build_skirmish_shell_instances, text};
use crate::render::batch::{BatchRenderer, SpriteInstance};
use crate::render::bit_font::BitFont;
use crate::render::terrain_draw_gpu_tests::{Gpu, camera};
use crate::ui::skirmish_shell::{self, RectPx, SkirmishShellState};
use wgpu::util::DeviceExt;

fn differences(a: &[u8], b: &[u8], size: [u32; 2], rect: RectPx) -> usize {
    a.chunks_exact(4)
        .zip(b.chunks_exact(4))
        .enumerate()
        .filter(|(index, (a, b))| {
            a != b
                && rect.contains(
                    (*index as u32 % size[0]) as i32,
                    (*index as u32 / size[0]) as i32,
                )
        })
        .count()
}

#[test]
#[cfg(feature = "npatch")]
#[ignore = "requires VERA20K_AYMOD_DIR, retail shell assets and a real GPU; no window"]
fn aymod_tech_slider_option_layers_reach_gpu_at_native_and_scaled_size() {
    let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
    let mut assets = crate::assets::asset_manager::AssetManager::new(
        &root,
        crate::assets::asset_manager::MediaArchiveMode::STOCK_DIGITAL,
    )
    .unwrap();
    assets.register_neutral_archives().unwrap();
    let ini = crate::rules::ini_parser::IniFile::from_bytes(
        &std::fs::read(root.join("rulesmd.ini")).unwrap(),
    )
    .unwrap();
    let rules = crate::rules::ruleset::RuleSet::from_ini(&ini).unwrap();
    let csf = crate::app::loading::init::load_csf(&assets).unwrap();
    assert!(
        csf.get("GUI:TechLevel")
            .is_some_and(|value| !value.is_empty())
    );
    assert!(
        csf.get("STT:SkirmishSliderTech")
            .is_some_and(|value| !value.is_empty())
    );
    let fnt =
        crate::assets::fnt_file::FntFile::from_bytes(assets.get_ref("GAME.FNT").unwrap()).unwrap();
    for codepoint in csf.get("GUI:TechLevel").unwrap().encode_utf16() {
        assert!(
            fnt.glyph(codepoint).is_some(),
            "actual tech label must have a real font glyph: {codepoint:X}"
        );
    }
    let gpu = Gpu::new();
    for (format, size, name) in [
        (wgpu::TextureFormat::Bgra8UnormSrgb, [800, 600], "native"),
        (wgpu::TextureFormat::Rgba8UnormSrgb, [1280, 720], "scaled"),
    ] {
        let batch = BatchRenderer::new_with_device(&gpu.device, &gpu.queue, format);
        batch.write_camera(&gpu.queue, camera(size));
        let atlas = crate::render::skirmish_shell_chrome::build_skirmish_shell_chrome_atlas(
            &gpu.device,
            &gpu.queue,
            &batch,
            &assets,
        )
        .expect("the actual shell chrome must load");
        let font = BitFont::from_fnt(&gpu.device, &gpu.queue, &batch, &fnt);
        let layout = if name == "native" {
            skirmish_shell::compute_layout(size[0], size[1])
        } else {
            skirmish_shell::compute_responsive_layout(size[0], size[1])
        };
        let vertex = |instances: &[SpriteInstance]| {
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Actual shell option instances"),
                    contents: bytemuck::cast_slice(instances),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        };
        let render =
            |shell: &SkirmishShellState,
             phase: &str,
             schemes: &[crate::rules::color_scheme::ColorSchemeEntry]| {
                let instances = build_skirmish_shell_instances(
                    &atlas,
                    &font,
                    &layout,
                    None,
                    None,
                    None,
                    shell,
                    schemes,
                    rules.npatch_dropdown_colors,
                    &[],
                    None,
                    false,
                );
                let chrome_buffer = vertex(&instances);
                let mut draws = Vec::new();
                text::push_option_text_draws(
                    &mut draws,
                    &font,
                    &layout,
                    shell,
                    |key, _fallback| csf.text(key).into_owned(),
                );
                let buffers = draws
                    .iter()
                    .map(|draw| (!draw.instances.is_empty()).then(|| vertex(&draw.instances)))
                    .collect::<Vec<_>>();
                let color = gpu.target(size, format);
                let depth = gpu.target(size, wgpu::TextureFormat::Depth32Float);
                let cv = color.create_view(&Default::default());
                let dv = depth.create_view(&Default::default());
                let mut encoder = gpu.device.create_command_encoder(&Default::default());
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Actual skirmish option layers"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &cv,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &dv,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    batch.draw_with_buffer_passthrough(
                        &mut pass,
                        &atlas.texture,
                        &chrome_buffer,
                        instances.len() as u32,
                    );
                    for (draw, buffer) in draws.iter().zip(&buffers) {
                        if let Some(buffer) = buffer {
                            pass.set_scissor_rect(
                                draw.scissor.x,
                                draw.scissor.y,
                                draw.scissor.w,
                                draw.scissor.h,
                            );
                            batch.draw_with_buffer_passthrough(
                                &mut pass,
                                font.atlas(),
                                buffer,
                                draw.instances.len() as u32,
                            );
                        }
                    }
                }
                let reads = [
                    gpu.read(&mut encoder, &color),
                    gpu.read(&mut encoder, &depth),
                ];
                let output = gpu.finish(encoder, &reads, size);
                assert!(
                    output[1]
                        .chunks_exact(4)
                        .all(|pixel| pixel == 1.0f32.to_le_bytes()),
                    "UI stores must leave depth unchanged"
                );
                if let Some(directory) = std::env::var_os("VERA20K_TECH_SLIDER_CAPTURE_DIR") {
                    let directory = std::path::PathBuf::from(directory);
                    std::fs::create_dir_all(&directory).unwrap();
                    let mut rgba = output[0].clone();
                    if format == wgpu::TextureFormat::Bgra8UnormSrgb {
                        for pixel in rgba.chunks_exact_mut(4) {
                            pixel.swap(0, 2);
                        }
                    }
                    image::save_buffer(
                        directory.join(format!("tech-{name}-{phase}.png")),
                        &rgba,
                        size[0],
                        size[1],
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                }
                output.into_iter().next().unwrap()
            };
        let mut shell = SkirmishShellState::default();
        shell.trackbar_bounds =
            skirmish_shell::SkirmishTrackbarBounds::from_multiplayer_dialog_settings(&ini);
        shell.apply_multiplayer_dialog_values(
            &crate::sim::game_options::GameOptions::from_multiplayer_dialog_settings(&ini),
        );
        shell.apply_ui_settings(&ini);
        assert_eq!(shell.launch_options_base.tech_level, 10);
        let ten = render(&shell, "10", &rules.color_schemes);
        let rect = layout.trackbars.tech_level;
        skirmish_shell::handle_option_mouse_down(
            &mut shell,
            &layout,
            &[],
            rect.x + (7.0 * layout.scale.factor()).round() as i32,
            rect.y + rect.h - (17.0 * layout.scale.factor()).round() as i32,
        );
        assert_eq!(shell.launch_options_base.tech_level, 1);
        skirmish_shell::handle_option_mouse_up(&mut shell);
        let one = render(&shell, "1", &rules.color_schemes);
        let value = crate::ui::shell::trackbar::value_text_rect(rect, layout.scale);
        assert!(
            differences(&ten, &one, size, value) > 0,
            "the real numeric glyphs must change, not only the thumb"
        );
        shell.apply_ui_settings(&crate::rules::ini_parser::IniFile::from_str(
            "[UISettings]\nShowTechLevelSlider=no\n",
        ));
        let hidden = render(&shell, "hidden", &rules.color_schemes);
        assert!(differences(&one, &hidden, size, layout.trackbar_labels.tech_level) > 0);
        assert!(differences(&one, &hidden, size, rect) > 0);
        // ShellTrackbar61E1B9..61E269 uses6208F0's two-authored-pixel
        // outside bevel. Its painted bounds exceed the interactive window.
        let base = layout.scale.base_rect(rect);
        let frame = atlas
            .control_chrome()
            .trackbar_frame(true, base.w, base.h)
            .unwrap();
        let painted = layout.scale.scale_rect(RectPx::new(
            base.x - 2,
            base.y - 2,
            frame.pixel_size[0] as i32,
            frame.pixel_size[1] as i32,
        ));
        for (index, (a, b)) in one.chunks_exact(4).zip(hidden.chunks_exact(4)).enumerate() {
            if a != b {
                let (x, y) = (
                    (index as u32 % size[0]) as i32,
                    (index as u32 / size[0]) as i32,
                );
                assert!(
                    painted.contains(x, y) || layout.trackbar_labels.tech_level.contains(x, y),
                    "hiding tech changed unrelated pixels at{x},{y}"
                );
            }
        }
        let checkbox = layout
            .checkboxes
            .iter()
            .find(|checkbox| checkbox.id == skirmish_shell::SkirmishCheckboxId::BuildOffAlly0x69d)
            .unwrap();
        let old = shell.build_off_ally;
        skirmish_shell::handle_option_mouse_down(
            &mut shell,
            &layout,
            &[],
            checkbox.rect.x + 3,
            checkbox.rect.y + 3,
        );
        assert_ne!(shell.build_off_ally, old);
        let toggled = render(&shell, "ally-toggled", &rules.color_schemes);
        assert!(
            differences(&hidden, &toggled, size, checkbox.rect) > 0,
            "the relocated actual checkbox must draw and respond"
        );
        // The same retail emitter must paint MPC16, not the old grey
        // out-of-range swatch. Compare it with MPC01 carrying that exact HSV.
        shell.refresh_country_catalog(&rules);
        shell.player_color_index = 15;
        let last = render(&shell, "mpc16", &rules.color_schemes);
        let mut alias = rules.color_schemes.clone();
        let first = alias
            .iter()
            .position(|entry| entry.name == "MPC01")
            .unwrap();
        let last_entry = alias
            .iter()
            .position(|entry| entry.name == "MPC16")
            .unwrap();
        alias[first].hsv = alias[last_entry].hsv;
        shell.player_color_index = 0;
        let reference = render(&shell, "mpc16-reference", &alias);
        let swatch = skirmish_shell::combo_swatch_rect(layout.color_combos[0], layout.scale);
        assert_eq!(differences(&last, &reference, size, swatch), 0);
        shell.player_color_claimed = false;
        let grey = render(&shell, "unclaimed", &rules.color_schemes);
        assert!(differences(&last, &grey, size, swatch) > 0);
    }
}

//! The real My Information color rows, profile owner and child painter, without
//! a window or WOL connection. No replacement palette, shader or font.

use super::*;
use crate::app::frontend::skirmish_shell_render::paint_wol_my_info_children;
use crate::render::batch::{BatchRenderer, SpriteInstance};
use crate::render::bit_font::BitFont;
use crate::render::skirmish_shell_chrome::SkirmishShellChromeAtlas;
use crate::render::terrain_draw_gpu_tests::{Gpu, camera};
use crate::rules::ruleset::RuleSet;
use crate::ui::shell::list::ListScrollPart;
use wgpu::util::DeviceExt;

fn render(
    gpu: &Gpu,
    batch: &BatchRenderer,
    atlas: &SkirmishShellChromeAtlas,
    font: &BitFont,
    rules: &RuleSet,
    page: &WolMyInfoPage,
    layout: &WolMyInfoLayout,
    size: [u32; 2],
    format: wgpu::TextureFormat,
    label: impl Fn(&str) -> String,
) -> Vec<u8> {
    let paint = paint_wol_my_info_children(Some(atlas), font, Some(rules), page, layout, label);
    let vertex = |instances: &[SpriteInstance]| {
        gpu.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Real WOL My Information children"),
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX,
            })
    };
    let controls = vertex(&paint.controls);
    let popup = vertex(&paint.popup);
    let text_buffers = paint
        .texts
        .iter()
        .chain(&paint.popup_texts)
        .map(|draw| (!draw.instances.is_empty()).then(|| vertex(&draw.instances)))
        .collect::<Vec<_>>();
    let color = gpu.target(size, format);
    let depth = gpu.target(size, wgpu::TextureFormat::Depth32Float);
    let cv = color.create_view(&Default::default());
    let dv = depth.create_view(&Default::default());
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Real WOL children and popup"),
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
            &controls,
            paint.controls.len() as u32,
        );
        for (index, draw) in paint.texts.iter().chain(&paint.popup_texts).enumerate() {
            if index == paint.texts.len() {
                pass.set_scissor_rect(0, 0, size[0], size[1]);
                batch.draw_with_buffer_passthrough(
                    &mut pass,
                    &atlas.texture,
                    &popup,
                    paint.popup.len() as u32,
                );
            }
            if let Some(buffer) = &text_buffers[index] {
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
        // A colors-only popup has no text rows after scrolling to the end.
        if paint.popup_texts.is_empty() {
            pass.set_scissor_rect(0, 0, size[0], size[1]);
            batch.draw_with_buffer_passthrough(
                &mut pass,
                &atlas.texture,
                &popup,
                paint.popup.len() as u32,
            );
        }
    }
    let reads = [
        gpu.read(&mut encoder, &color),
        gpu.read(&mut encoder, &depth),
    ];
    let mut images = gpu.finish(encoder, &reads, size);
    assert!(
        images[1]
            .chunks_exact(4)
            .all(|pixel| pixel == 1.0f32.to_le_bytes()),
        "shell color and popup paint must not write depth"
    );
    images.remove(0)
}

#[test]
#[cfg(feature = "npatch")]
#[ignore = "requires VERA20K_AYMOD_DIR, retail UI assets and a real GPU; no window"]
fn aymod_wol_profile_color_sixteen_reaches_real_page_paint_after_reopen() {
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
    let rules = RuleSet::from_ini(&ini).unwrap();
    let csf = crate::app::loading::init::load_csf(&assets).unwrap();
    let label = |key: &str| csf.text(key).into_owned();
    let fnt =
        crate::assets::fnt_file::FntFile::from_bytes(assets.get_ref("GAME.FNT").unwrap()).unwrap();
    assert_eq!(rules.multiplayer_color_count, 16);
    assert_eq!(color_rows(None, |_| String::new()).len(), 9);
    let rows = || MyInfoRows {
        nicknames: vec![MyInfoRow::text("AYColor", 1)],
        default_text: label(DEFAULT_KEY),
        colors: color_rows(Some(&rules), |_| label("GUI:RandomAsSymbols")),
        ..Default::default()
    };
    assert_eq!(rows().colors.last().unwrap().swatch, Some(15));
    let gpu = Gpu::new();
    for (format, size) in [
        (wgpu::TextureFormat::Bgra8UnormSrgb, [800, 600]),
        (wgpu::TextureFormat::Rgba8UnormSrgb, [1280, 720]),
    ] {
        let batch = BatchRenderer::new_with_device(&gpu.device, &gpu.queue, format);
        batch.write_camera(&gpu.queue, camera(size));
        let atlas = crate::render::skirmish_shell_chrome::build_skirmish_shell_chrome_atlas(
            &gpu.device,
            &gpu.queue,
            &batch,
            &assets,
        )
        .unwrap();
        let font = BitFont::from_fnt(&gpu.device, &gpu.queue, &batch, &fnt);
        let layout = WolMyInfoLayout::responsive(size[0] as i32, size[1] as i32);
        let directory = crate::map::source::test_support::TestDirectory::new("ay-wol-color-gpu");
        let file =
            crate::app::persistence::wol_profile::WolInfoFile::in_directory(directory.path());
        let mut persona = file.load_persona("AYColor", &label);
        persona.set_color(0);
        let mut page = WolMyInfoPage::open(rows(), "AYColor", &view(&persona), Vec::new(), false);
        let face = layout.face(COLOR).unwrap();
        let pixel = |image: &[u8], x: i32, y: i32| {
            let offset = ((y as u32 * size[0] + x as u32) * 4) as usize;
            image[offset..offset + 4].to_vec()
        };
        let closed = render(
            &gpu, &batch, &atlas, &font, &rules, &page, &layout, size, format, label,
        );
        page.popup.open(COLOR, page.combo(COLOR).unwrap().selected);
        let popup = page.popup_geometry(&layout).unwrap();
        let thumb = popup.thumb().unwrap();
        let scrollbar = popup.scrollbar().unwrap();
        page.popup.scroll.press(
            ListScrollPart::Thumb,
            popup.scroll_geometry(),
            &mut page.popup.top_index,
            thumb.y + 1,
            Instant::now(),
        );
        page.popup
            .pointer_moved(popup, thumb.x + 1, scrollbar.y + scrollbar.h);
        page.popup.cancel_pointer();
        let popup = page.popup_geometry(&layout).unwrap();
        let last = popup
            .row_rect(16)
            .expect("the actual sixteenth color must be visible");
        let open = render(
            &gpu, &batch, &atlas, &font, &rules, &page, &layout, size, format, label,
        );
        let pick = popup.row_at(last.x + last.w / 2, last.y + 5).unwrap();
        assert_eq!(page.select(COLOR, pick), Some(MyInfoAction::Color(15)));
        persona.set_color(page.combo(COLOR).unwrap().data_or(-99));
        file.save_persona(&mut persona, "AYColor", &label);
        let saved = file.load_persona("AYColor", &label);
        assert_eq!(saved.color, 15);
        let reopened = WolMyInfoPage::open(rows(), "AYColor", &view(&saved), Vec::new(), false);
        assert_eq!(reopened.combo(COLOR).unwrap().data_or(-99), 15);
        let restored = render(
            &gpu, &batch, &atlas, &font, &rules, &reopened, &layout, size, format, label,
        );
        let face_pixel = |image: &[u8]| pixel(image, face.x + face.w / 2, face.y + 5);
        assert_ne!(face_pixel(&restored), face_pixel(&closed));
        assert_eq!(
            face_pixel(&restored),
            pixel(&open, last.x + last.w / 2, last.y + 5),
            "the scrolled popup and restored folded face must draw the same MPC16 color"
        );
        if let Some(directory) = std::env::var_os("VERA20K_WOL_COLOR_CAPTURE_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            for (name, mut rgba) in [("popup", open), ("reopened", restored)] {
                if format == wgpu::TextureFormat::Bgra8UnormSrgb {
                    for pixel in rgba.chunks_exact_mut(4) {
                        pixel.swap(0, 2);
                    }
                }
                image::save_buffer(
                    directory.join(format!("wol-{}-{name}.png", size[0])),
                    &rgba,
                    size[0],
                    size[1],
                    image::ColorType::Rgba8,
                )
                .unwrap();
            }
        }
    }
}

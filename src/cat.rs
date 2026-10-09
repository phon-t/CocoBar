use image::imageops::FilterType;
use image::{load_from_memory_with_format, ImageFormat};
use rand::RngExt;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::time::Instant;
use windows_sys::Win32::Foundation::{HWND, POINT, SIZE};
use windows_sys::Win32::Graphics::Gdi::{
    CreateDIBSection, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, RGBQUAD,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, UpdateLayeredWindow, ULW_ALPHA};

pub(crate) const CONTENT_X: i32 = 364;
pub(crate) const CONTENT_Y: i32 = 148;
pub(crate) const CONTENT_W: i32 = 1192;
pub(crate) const CONTENT_H: i32 = 1748;
// Tilted cat occupies a wider rotated box on the same 2000x2000 canvas
pub(crate) const TILT_X: i32 = 84;
pub(crate) const TILT_Y: i32 = 63;
pub(crate) const TILT_W: i32 = 1858;
pub(crate) const TILT_H: i32 = 1834;
// The tail reaches below the cropped artwork. Keep that space in the window,
// rather than clipping the last segment against the asset's crop rectangle.
pub(crate) const TILT_CANVAS_H: i32 = 1980;
// The raw tilted canvas reads bigger than the normal one - draw it a touch smaller
pub(crate) const TILT_SCALE_FACTOR: f32 = 0.90;

pub(crate) fn tilted_scale(normal_scale: f32) -> f32 {
    (TILT_W as f32 * normal_scale * TILT_SCALE_FACTOR)
        .round()
        .max(1.0)
        / TILT_W as f32
}

// --- Cartoon tail (attached at the user's pink mark on the tilted cat's butt) ---
// Anchor is in TILT-layer coords. The pink mark (513,1714) is 240px INSIDE the body,
// which hid the whole base of the tail behind the body (cut/detached look). The
// anchor is moved LEFT to the body's visible left edge so the tail emerges from the
// butt and hangs down to the floor where it rests.
const TAIL_SEGMENTS: usize = 8;
const TAIL_REST: f32 = 28.0;
const TAIL_BASE_R: f32 = 42.0;
const TAIL_TIP_R: f32 = 56.0; // wider rounded cartoon end
const TAIL_BORDER_W: f32 = 16.0; // matching asset border thickness
const TAIL_DAMP: f32 = 0.88; // responsive springy cartoon sway
const TAIL_GRAV: f32 = 1100.0; // weight pulling downward under gravity
const TAIL_MAX_SPEED: f32 = 140.0; // canvas px/frame velocity clamp
const TAIL_GROUND_Y: f32 = TILT_CANVAS_H as f32 - 8.0;
const TAIL_AX: f32 = 470.0; // shifted slightly right on the butt
const TAIL_AY: f32 = 1690.0;
pub(crate) const MAX_OFF_X: f32 = 85.0;
pub(crate) const MAX_OFF_Y: f32 = 100.0;
pub(crate) const DEADZONE: f32 = 20.0;

const BLACK_SOCKET: &[u8] = include_bytes!("../assets/blackcatwitheyesocket.png");
const WHITE_SOCKET: &[u8] = include_bytes!("../assets/whitecatwitheyesocket.png");
const ORANGE_SOCKET: &[u8] = include_bytes!("../assets/orangecatwitheyesocket.png");
const BLACK_CLOSED: &[u8] = include_bytes!("../assets/blackcateyesclossed.png");
const WHITE_CLOSED: &[u8] = include_bytes!("../assets/whitecateyesclossed.png");
const ORANGE_CLOSED: &[u8] = include_bytes!("../assets/orangecateyesclossed.png");
const EYES: &[u8] = include_bytes!("../assets/eyes.png");
const HIGHLIGHT: &[u8] = include_bytes!("../assets/highlight.png");
const BLACK_ANNOYED: &[u8] = include_bytes!("../assets/blackcatannoyed.png");
const WHITE_ANNOYED: &[u8] = include_bytes!("../assets/whitecatannoyed.png");
const ORANGE_ANNOYED: &[u8] = include_bytes!("../assets/orangecatannoyed.png");

const TILTED_SOCKETS: [&[u8]; 3] = [
    include_bytes!("../assets/Cat tilted/tiltedcatblackwitheyesocket.png"),
    include_bytes!("../assets/Cat tilted/tiltedcatwhitewitheyesocket.png"),
    include_bytes!("../assets/Cat tilted/tiltedcatorangewitheyesocket.png"),
];
const TILTED_PUPIL: &[u8] = include_bytes!("../assets/Cat tilted/tiltedcatpupil.png");
const SPIRAL_LEFT: &[u8] = include_bytes!("../assets/spiralpupilleft.png");
const SPIRAL_RIGHT: &[u8] = include_bytes!("../assets/spiralpupilright.png");

const TILTED_SCARF_ITEMS: &[super::cosmetics::CosmeticItem] = &[
    super::cosmetics::CosmeticItem {
        name: "Tilted Black Scarf",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedscarf/tiltedscarfblack.png"),
    },
    super::cosmetics::CosmeticItem {
        name: "Tilted Red Scarf",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedscarf/tiltedscarfred.png"),
    },
    super::cosmetics::CosmeticItem {
        name: "Tilted Blue Scarf",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedscarf/tiltedscarfblue.png"),
    },
    super::cosmetics::CosmeticItem {
        name: "Tilted Pink Scarf",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedscarf/tiltedscarfpink.png"),
    },
    super::cosmetics::CosmeticItem {
        name: "Tilted White Scarf",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedscarf/tiltedscarfwhite.png"),
    },
];
const TILTED_BELL_ITEMS: &[super::cosmetics::CosmeticItem] = &[
    super::cosmetics::CosmeticItem {
        name: "Tilted Black Bell",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedbell/tiltedbellblack.png"),
    },
    super::cosmetics::CosmeticItem {
        name: "Tilted Red Bell",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedbell/tiltedbellred.png"),
    },
];
const TILTED_TIE_ITEMS: &[super::cosmetics::CosmeticItem] = &[
    super::cosmetics::CosmeticItem {
        name: "Tilted Blue Tie",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedtie/tiltedtieblue.png"),
    },
    super::cosmetics::CosmeticItem {
        name: "Tilted Orange Tie",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedtie/tiltedtieorange.png"),
    },
    super::cosmetics::CosmeticItem {
        name: "Tilted Red Tie",
        bytes: include_bytes!("../assets/Cat tilted cosmetics/tiltedtie/tiltedtiered.png"),
    },
];

#[derive(Clone)]
pub(crate) struct Layer {
    pub w: usize,
    pub h: usize,
    pub data: Vec<u8>,
}

#[cfg(test)]
mod animation_tests {
    use super::*;
    use std::time::Duration;

    fn renderer() -> CatRenderer {
        let mut cat = CatRenderer::new();
        cat.blink_next = Instant::now() + Duration::from_secs(3600);
        cat.rebuild_layers(0, 180, 264, 180.0 / CONTENT_W as f32);
        cat
    }

    fn frame(cat: &mut CatRenderer, now: Instant, cursor: POINT, tilted: bool) -> (usize, usize) {
        let scale = if tilted {
            tilted_scale(cat.built_scale)
        } else {
            cat.built_scale
        };
        let (w, h) = if tilted {
            (
                (TILT_W as f32 * scale).round() as i32,
                (TILT_CANVAS_H as f32 * scale).round() as i32,
            )
        } else {
            (cat.base.w as i32, cat.base.h as i32)
        };
        cat.compose_frame(0, 0, w, h, scale, cursor, now);
        assert_eq!(cat.buf.len(), (w * h * 4) as usize);
        assert!(
            cat.buf.chunks_exact(4).filter(|p| p[3] > 0).count() > (w * h / 4) as usize,
            "The current pose must contain a visible cat"
        );
        (w as usize, h as usize)
    }

    fn save_frame(cat: &CatRenderer, size: (usize, usize), name: &str) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/animation-check");
        std::fs::create_dir_all(&dir).unwrap();
        let mut rgba = cat.buf.clone();
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            if pixel[3] > 0 {
                for channel in 0..3 {
                    pixel[channel] =
                        ((pixel[channel] as u32 * 255) / pixel[3] as u32).min(255) as u8;
                }
            }
        }
        image::save_buffer(
            dir.join(format!("{name}.png")),
            &rgba,
            size.0 as u32,
            size.1 as u32,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }

    fn assert_outfit(cat: &CatRenderer, size: (usize, usize), tilted: bool) {
        let layers = if tilted {
            [&cat.tilt_scarf, &cat.tilt_bell, &cat.tilt_tie]
        } else {
            [&cat.scarf, &cat.bell, &cat.tie]
        };
        let mut expected = vec![0; cat.buf.len()];
        for layer in layers.into_iter().flatten() {
            blit(
                &mut expected,
                size.0,
                size.1,
                &layer.data,
                layer.w,
                layer.h,
                0,
                0,
            );
        }
        let mut visible = 0;
        for (outfit, actual) in expected.chunks_exact(4).zip(cat.buf.chunks_exact(4)) {
            if outfit[3] == 255 {
                visible += 1;
                assert_eq!(
                    outfit, actual,
                    "Opaque outfit pixels must survive the reaction"
                );
            }
        }
        assert!(
            visible > 0,
            "Selected accessories must render visible pixels"
        );
    }

    #[test]
    fn normal_reactions_for_all_colors_preserve_accessories() {
        let mut cat = renderer();
        let start = Instant::now();
        let center = POINT { x: 90, y: 132 };
        for (color, name) in ["black", "white", "orange"].iter().enumerate() {
            let now = start + Duration::from_secs(color as u64 * 10);
            cat.rebuild_layers(color, 180, 264, cat.built_scale);
            cat.rebuild_cosmetics(Some(1), Some(1), None, cat.built_scale);
            cat.off_x = 0.0;
            cat.off_y = 0.0;
            let size = frame(&mut cat, now, center, false);
            let normal = cat.buf.clone();
            assert_outfit(&cat, size, false);
            save_frame(&cat, size, &format!("{name}-normal"));
            frame(
                &mut cat,
                now + Duration::from_millis(16),
                POINT { x: 900, y: 132 },
                false,
            );
            assert!(cat.off_x > 0.0, "Eyes follow a cursor to the right");
            for i in 2..32 {
                frame(
                    &mut cat,
                    now + Duration::from_millis(i * 16),
                    POINT { x: -900, y: 132 },
                    false,
                );
            }
            assert!(cat.off_x < 0.0, "Eyes follow a cursor to the left");
            assert!(
                cat.off_x.hypot(cat.off_y) <= MAX_OFF_Y,
                "Pupils stay inside their sockets"
            );
            save_frame(&cat, size, &format!("{name}-tracking"));
            cat.off_x = 0.0;
            cat.off_y = 0.0;
            cat.blink_start = Some(now + Duration::from_secs(1));
            frame(&mut cat, now + Duration::from_millis(1130), center, false);
            assert_ne!(
                cat.buf, normal,
                "The closed-eye frame differs from the open-eye frame"
            );
            assert_outfit(&cat, size, false);
            save_frame(&cat, size, &format!("{name}-blink"));
            frame(&mut cat, now + Duration::from_millis(1300), center, false);
            assert!(cat.blink_start.is_none());
            assert_eq!(cat.buf, normal, "A blink returns to the normal face");
            cat.set_look_down();
            cat.look_down_start = Some(now + Duration::from_secs(2));
            frame(&mut cat, now + Duration::from_millis(2700), center, false);
            assert_ne!(
                cat.buf, normal,
                "Applying an accessory produces a downward glance"
            );
            assert_outfit(&cat, size, false);
            save_frame(&cat, size, &format!("{name}-glance"));
            frame(&mut cat, now + Duration::from_millis(3100), center, false);
            assert!(cat.look_down_start.is_none());
            assert_eq!(
                cat.buf, normal,
                "The accessory glance returns to normal tracking"
            );
            assert!(cat.compose_annoyed(size.0, size.1));
            assert_ne!(cat.buf, normal, "Click feedback renders an annoyed face");
            assert_outfit(&cat, size, false);
            save_frame(&cat, size, &format!("{name}-annoyed"));
            frame(&mut cat, now + Duration::from_secs(4), center, false);
            assert_eq!(
                cat.buf, normal,
                "Normal rendering restores the face after the click reaction"
            );
            cat.drag_active = true;
            let tilted_size = frame(&mut cat, now + Duration::from_secs(5), center, true);
            assert_outfit(&cat, tilted_size, true);
            save_frame(&cat, tilted_size, &format!("{name}-tilted"));
            cat.drag_active = false;
            frame(&mut cat, now + Duration::from_secs(6), center, false);
            assert_eq!(
                cat.buf, normal,
                "Each cat color returns from the tilted pose"
            );
        }
    }

    #[test]
    fn dragging_dizzy_recovery_and_slow_frames() {
        let mut cat = renderer();
        cat.rebuild_cosmetics(Some(1), Some(1), None, cat.built_scale);
        let now = Instant::now();
        let center = POINT { x: 90, y: 132 };
        frame(&mut cat, now, center, false);
        let normal = cat.buf.clone();
        cat.drag_active = true;
        let size = frame(&mut cat, now + Duration::from_millis(16), center, true);
        assert!(cat.tail.ready, "Dragging starts the hanging tail");
        assert!(cat.tilt_pupil.is_some());
        assert_outfit(&cat, size, true);
        save_frame(&cat, size, "drag-tilted");
        for i in 1..=4 {
            frame(
                &mut cat,
                now + Duration::from_millis(16 + i * 100),
                POINT {
                    x: 90 + i as i32 * 25,
                    y: 132,
                },
                true,
            );
        }
        assert!(
            !cat.fast_drag,
            "A slow drag stays calm even with delayed frames"
        );
        cat.drag_active = false;
        frame(&mut cat, now + Duration::from_millis(432), center, false);
        assert!(
            cat.spiral_start.is_none(),
            "An ordinary drop does not trigger dizziness"
        );
        cat.drag_active = true;
        frame(&mut cat, now + Duration::from_millis(448), center, true);
        for i in 1..=4 {
            frame(
                &mut cat,
                now + Duration::from_millis(448 + i * 16),
                POINT {
                    x: 90 + i as i32 * 40,
                    y: 132,
                },
                true,
            );
        }
        assert!(
            cat.fast_drag,
            "Sustained fast movement arms the dizzy reaction"
        );
        cat.drag_active = false;
        let size = frame(&mut cat, now + Duration::from_millis(528), center, false);
        assert!(cat.spiral_start.is_some());
        assert_eq!(cat.spiral_left.as_ref().unwrap().len(), 12);
        assert_ne!(cat.buf, normal);
        assert_outfit(&cat, size, false);
        save_frame(&cat, size, "drag-dizzy");
        let expiry = cat.spiral_start.unwrap() + Duration::from_secs_f32(cat.spiral_dur + 0.4);
        cat.off_x = 0.0;
        cat.off_y = 0.0;
        frame(&mut cat, expiry, center, false);
        assert!(
            cat.spiral_start.is_none(),
            "Dizzy eyes expire after their fade"
        );
        assert_eq!(
            cat.buf, normal,
            "Dropping restores the normal canvas and outfit"
        );
        cat.spiral_start = Some(expiry);
        cat.drag_active = true;
        frame(&mut cat, expiry + Duration::from_millis(16), center, true);
        assert!(
            cat.spiral_start.is_none(),
            "A fresh grab clears the previous dizzy reaction"
        );
        cat.drag_active = false;
        frame(&mut cat, expiry + Duration::from_millis(32), center, false);
        assert!(cat.spiral_start.is_none());
    }

    #[test]
    fn every_accessory_renders_in_normal_and_tilted_poses_after_resize() {
        let mut cat = renderer();
        let now = Instant::now();
        let outfits = (0..5)
            .map(|i| (None, Some(i), None))
            .chain((0..2).map(|i| (Some(i), None, None)))
            .chain((0..3).map(|i| (None, None, Some(i))));
        for (i, (bell, scarf, tie)) in outfits.enumerate() {
            let width = if i % 2 == 0 { 120 } else { 200 };
            let scale = width as f32 / CONTENT_W as f32;
            cat.rebuild_layers(0, width, (CONTENT_H as f32 * scale).round() as i32, scale);
            cat.rebuild_cosmetics(bell, scarf, tie, scale);
            cat.drag_active = false;
            let t = now + Duration::from_secs(i as u64 * 2);
            let center_y = cat.base.h as i32 / 2;
            let size = frame(
                &mut cat,
                t,
                POINT {
                    x: width / 2,
                    y: center_y,
                },
                false,
            );
            assert_outfit(&cat, size, false);
            cat.drag_active = true;
            let size = frame(
                &mut cat,
                t + Duration::from_millis(16),
                POINT { x: 0, y: 0 },
                true,
            );
            assert_outfit(&cat, size, true);
        }
        cat.rebuild_cosmetics(None, None, None, cat.built_scale);
        cat.ensure_tilted(tilted_scale(cat.built_scale));
        assert!(cat.scarf.is_none() && cat.bell.is_none() && cat.tie.is_none());
        assert!(cat.tilt_scarf.is_none() && cat.tilt_bell.is_none() && cat.tilt_tie.is_none());
    }

    #[test]
    fn idle_throttling_keeps_reactions_animating() {
        let mut cat = CatRenderer::new();
        cat.blink_next = Instant::now() + Duration::from_secs(60);
        cat.cursor_still = 2.0;
        assert_eq!((0..32).filter(|_| cat.tick(false)).count(), 4);
        assert!(cat.tick(true), "Cursor movement immediately wakes tracking");
        cat.cursor_still = 2.0;
        cat.blink_start = Some(Instant::now());
        assert!(cat.tick(false));
        cat.blink_start = None;
        cat.set_look_down();
        assert!(cat.tick(false));
        cat.look_down_start = None;
        cat.drag_active = true;
        assert!(cat.tick(false));
        cat.drag_active = false;
        cat.spiral_start = Some(Instant::now());
        assert!(cat.tick(false));
    }

    #[test]
    fn tail_trails_the_drag_settles_and_keeps_its_tip_inside_the_canvas() {
        let mut tail = Tail::new();
        for _ in 0..240 {
            tail.update(TAIL_AX, TAIL_AY, 0.0, 0.0, 0.016);
        }
        let resting_x = tail.pts.last().unwrap().x;
        tail.update(TAIL_AX, TAIL_AY, 40.0, 0.0, 0.016);
        assert!(
            tail.pts.last().unwrap().x < resting_x,
            "Moving right makes the tail trail to the left"
        );
        for i in 0..720 {
            let delta = if i < 360 {
                if i % 2 == 0 {
                    600.0
                } else {
                    -600.0
                }
            } else {
                0.0
            };
            let dt = if i % 3 == 0 { 0.032 } else { 0.008 };
            tail.update(TAIL_AX, TAIL_AY, delta, delta / 3.0, dt);
            assert!((tail.pts[0].x - TAIL_AX).abs() < 0.001);
            assert!((tail.pts[0].y - TAIL_AY).abs() < 0.001);
            for (index, point) in tail.pts.iter().enumerate() {
                assert!(point.x.is_finite() && point.y.is_finite());
                let radius = TAIL_BASE_R
                    + (TAIL_TIP_R - TAIL_BASE_R) * index as f32 / (TAIL_SEGMENTS - 1) as f32
                    + TAIL_BORDER_W;
                assert!(point.x - radius >= 0.0 && point.x + radius < TILT_W as f32);
                assert!(
                    point.y + radius < TILT_CANVAS_H as f32,
                    "The entire tail tip and border fit inside the window"
                );
            }
            for points in tail.pts.windows(2) {
                assert!(
                    (points[1].x - points[0].x).hypot(points[1].y - points[0].y) < TAIL_REST * 1.2,
                    "Fast movement must not stretch the tail segments apart"
                );
            }
        }
        assert!(
            (tail.pts.last().unwrap().x - TAIL_AX).abs() < 10.0,
            "The tail settles after the drag stops"
        );
    }
}

impl Layer {
    pub(crate) fn new(w: usize, h: usize) -> Self {
        Layer {
            w,
            h,
            data: vec![0; w * h * 4],
        }
    }
}

pub(crate) fn rgba_to_layer(rgba: Vec<u8>, w: usize, h: usize) -> Layer {
    let mut l = Layer::new(w, h);
    let mut i = 0usize;
    while i + 3 < rgba.len() {
        let r = rgba[i] as u32;
        let g = rgba[i + 1] as u32;
        let b = rgba[i + 2] as u32;
        let a = rgba[i + 3] as u32;
        l.data[i] = (b * a / 255) as u8;
        l.data[i + 1] = (g * a / 255) as u8;
        l.data[i + 2] = (r * a / 255) as u8;
        l.data[i + 3] = a as u8;
        i += 4;
    }
    l
}

fn load_cos_layer(
    cache: &mut Vec<u8>,
    idx: &mut Option<usize>,
    want: Option<usize>,
    items: &[super::cosmetics::CosmeticItem],
    crop: (i32, i32, i32, i32),
    scale: f32,
) -> Option<Layer> {
    let item = want.and_then(|i| items.get(i))?;
    if *idx != want || cache.is_empty() {
        let (rgba, w, h) = decode_and_crop(item.bytes, crop);
        *cache = rgba;
        *idx = want;
        debug_assert_eq!((w, h), (crop.2 as u32, crop.3 as u32));
    }
    Some(load_layer_cached(
        cache,
        crop.2 as u32,
        crop.3 as u32,
        scale,
    ))
}

fn rotate_layer_nearest(src: &Layer, angle_deg: f32, cx: f32, cy: f32) -> Layer {
    let w = src.w as i32;
    let h = src.h as i32;
    let rad = angle_deg.to_radians();
    let (cos, sin) = (rad.cos(), rad.sin());
    let mut out = Layer::new(src.w, src.h);
    let mut i = 0usize;
    for y in 0..h {
        for x in 0..w {
            let fx = x as f32 - cx;
            let fy = y as f32 - cy;
            let sx = fx * cos + fy * sin;
            let sy = -fx * sin + fy * cos;
            let px = (sx + cx).round() as i32;
            let py = (sy + cy).round() as i32;
            if px >= 0 && py >= 0 && px < w && py < h {
                let si = ((py * w + px) as usize) * 4;
                out.data[i..i + 4].copy_from_slice(&src.data[si..si + 4]);
            }
            i += 4;
        }
    }
    out
}

pub(crate) fn decode_and_crop(bytes: &[u8], crop: (i32, i32, i32, i32)) -> (Vec<u8>, u32, u32) {
    let img = load_from_memory_with_format(bytes, ImageFormat::Png)
        .expect("png decode")
        .to_rgba8();
    let cropped = image::imageops::crop_imm(
        &img,
        crop.0 as u32,
        crop.1 as u32,
        crop.2 as u32,
        crop.3 as u32,
    )
    .to_image();
    let (w, h) = cropped.dimensions();
    (cropped.into_raw(), w, h)
}

pub(crate) fn load_layer_cached(
    cached_rgba: &[u8],
    cached_w: u32,
    cached_h: u32,
    scale: f32,
) -> Layer {
    let dw = (cached_w as f32 * scale).round().max(1.0) as u32;
    let dh = (cached_h as f32 * scale).round().max(1.0) as u32;
    let img =
        image::ImageBuffer::<image::Rgba<u8>, &[u8]>::from_raw(cached_w, cached_h, cached_rgba)
            .unwrap();
    let resized = image::imageops::resize(&img, dw, dh, FilterType::Triangle);
    let (w, h) = resized.dimensions();
    rgba_to_layer(resized.into_raw(), w as usize, h as usize)
}

pub(crate) fn blit(
    dst: &mut [u8],
    dw: usize,
    dh: usize,
    src: &[u8],
    sw: usize,
    sh: usize,
    dx: i32,
    dy: i32,
) {
    let x0 = dx.max(0) as usize;
    let y0 = dy.max(0) as usize;
    let x1 = ((dx + sw as i32).min(dw as i32)).max(0) as usize;
    let y1 = ((dy + sh as i32).min(dh as i32)).max(0) as usize;
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let sx0 = (x0 as i32 - dx).max(0) as usize;
    let sy0 = (y0 as i32 - dy).max(0) as usize;
    for y in y0..y1 {
        let sy = sy0 + (y - y0);
        let row_d = y * dw;
        let row_s = sy * sw;
        for x in x0..x1 {
            let sx = sx0 + (x - x0);
            let di = (row_d + x) * 4;
            let si = (row_s + sx) * 4;
            let a = src[si + 3] as u32;
            if a == 0 {
                continue;
            }
            if a == 255 {
                dst[di..di + 4].copy_from_slice(&src[si..si + 4]);
            } else {
                let inv = 255 - a;
                dst[di] = (src[si] as u32 + dst[di] as u32 * inv / 255) as u8;
                dst[di + 1] = (src[si + 1] as u32 + dst[di + 1] as u32 * inv / 255) as u8;
                dst[di + 2] = (src[si + 2] as u32 + dst[di + 2] as u32 * inv / 255) as u8;
                dst[di + 3] = (src[si + 3] as u32 + dst[di + 3] as u32 * inv / 255) as u8;
            }
        }
    }
}

pub(crate) fn crossfade(buf: &mut [u8], src: &[u8], t: f32) {
    let tb = (t.clamp(0.0, 1.0) * 255.0) as u32;
    let inv = 255 - tb;
    let mut i = 0usize;
    while i + 3 < buf.len() {
        buf[i] = (buf[i] as u32 * inv / 255 + src[i] as u32 * tb / 255) as u8;
        buf[i + 1] = (buf[i + 1] as u32 * inv / 255 + src[i + 1] as u32 * tb / 255) as u8;
        buf[i + 2] = (buf[i + 2] as u32 * inv / 255 + src[i + 2] as u32 * tb / 255) as u8;
        buf[i + 3] = (buf[i + 3] as u32 * inv / 255 + src[i + 3] as u32 * tb / 255) as u8;
        i += 4;
    }
}

// ---- Cartoon tail: physics + rendering ----

#[derive(Clone, Copy)]
struct TailPt {
    x: f32,
    y: f32,
    px: f32, // previous x (for Verlet)
    py: f32, // previous y
}

pub(crate) struct Tail {
    pts: Vec<TailPt>,
    ready: bool,
    step_dt: f32,
}

impl Tail {
    fn new() -> Self {
        Self {
            pts: vec![
                TailPt {
                    x: 0.0,
                    y: 0.0,
                    px: 0.0,
                    py: 0.0
                };
                TAIL_SEGMENTS
            ],
            ready: false,
            step_dt: super::FRAME_MS as f32 / 4000.0,
        }
    }
    pub(crate) fn reset(&mut self) {
        self.ready = false;
        self.step_dt = super::FRAME_MS as f32 / 4000.0;
    }
    /// Verlet physics in TILT-layer coordinate space. Anchor (ax, ay) in layer coords.
    /// Incorporates window dragging velocity (win_dx, win_dy) to give drag momentum.
    fn update(&mut self, ax: f32, ay: f32, win_dx: f32, win_dy: f32, dt: f32) {
        let total_len = TAIL_REST * (TAIL_SEGMENTS - 1) as f32;
        if !self.ready {
            for i in 0..TAIL_SEGMENTS {
                let t = i as f32 / (TAIL_SEGMENTS - 1) as f32;
                // Hang downwards from the butt with a subtle initial curve
                let x = ax - t * 25.0;
                let y = ay + t * total_len;
                self.pts[i] = TailPt { x, y, px: x, py: y };
            }
            self.ready = true;
        } else if win_dx != 0.0 || win_dy != 0.0 {
            // Moving the window shifts free tail points in the opposite local
            // direction. Shift both current/previous positions to preserve their
            // velocity, and cap a teleport's impulse to avoid a violent snap.
            let distance = win_dx.hypot(win_dy).max(1.0);
            let amount = (TAIL_REST * 3.0 / distance).min(1.0);
            for i in 1..TAIL_SEGMENTS {
                self.pts[i].x -= win_dx * amount;
                self.pts[i].y -= win_dy * amount;
                self.pts[i].px -= win_dx * amount;
                self.pts[i].py -= win_dy * amount;
            }
        }
        const SUBSTEPS: usize = 4;
        let h = dt.clamp(0.001, 1.0 / 30.0) / SUBSTEPS as f32;
        let h2 = h * h;
        let damp = TAIL_DAMP.powf(h / (super::FRAME_MS as f32 / 1000.0));
        let grav = TAIL_GRAV * h2;
        let spd = TAIL_MAX_SPEED / SUBSTEPS as f32;
        for step in 0..SUBSTEPS {
            let time_ratio = if step == 0 { h / self.step_dt } else { 1.0 };
            // Pin anchor (segment 0) to butt
            self.pts[0].x = ax;
            self.pts[0].y = ay;
            self.pts[0].px = ax;
            self.pts[0].py = ay;

            // Verlet integration (gravity)
            for i in 1..TAIL_SEGMENTS {
                let p = &mut self.pts[i];
                let mut vx = (p.x - p.px) * damp * time_ratio;
                let mut vy = (p.y - p.py) * damp * time_ratio;
                let sp = (vx * vx + vy * vy).sqrt();
                if sp > spd {
                    let k = spd / sp;
                    vx *= k;
                    vy *= k;
                }
                p.px = p.x;
                p.py = p.y;
                p.x += vx;
                p.y += vy + grav;
            }

            // Distance constraints (stiff chain for cartoon rope physics)
            for _ in 0..12 {
                self.pts[0].x = ax;
                self.pts[0].y = ay;
                for i in 0..TAIL_SEGMENTS - 1 {
                    let (px0, py0) = (self.pts[i].x, self.pts[i].y);
                    let (px1, py1) = (self.pts[i + 1].x, self.pts[i + 1].y);
                    let dx = px1 - px0;
                    let dy = py1 - py0;
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    let diff = (d - TAIL_REST) / d;
                    let ox = dx * diff * 0.5;
                    let oy = dy * diff * 0.5;
                    if i > 0 {
                        self.pts[i].x += ox;
                        self.pts[i].y += oy;
                    }
                    self.pts[i + 1].x -= if i == 0 { ox * 2.0 } else { ox };
                    self.pts[i + 1].y -= if i == 0 { oy * 2.0 } else { oy };
                }
                self.pts[0].x = ax;
                self.pts[0].y = ay;
            }

            // Leave a margin for the tip's border and antialiasing.
            for i in 1..TAIL_SEGMENTS {
                let t = i as f32 / (TAIL_SEGMENTS - 1) as f32;
                let r = TAIL_BASE_R + (TAIL_TIP_R - TAIL_BASE_R) * t + TAIL_BORDER_W;
                let ground = TAIL_GROUND_Y - r;
                if self.pts[i].y > ground {
                    self.pts[i].y = ground;
                    if self.pts[i].py > self.pts[i].y {
                        self.pts[i].py = self.pts[i].y;
                    }
                }
            }
        }
        self.step_dt = h;
    }
}

fn fill_circle(buf: &mut [u8], bw: usize, bh: usize, cx: i32, cy: i32, r: i32, c: [u8; 4]) {
    if r <= 0 || bw == 0 || bh == 0 {
        return;
    }
    if (cx as i32).is_negative() && cx < -r {
        return;
    }
    if (cy as i32).is_negative() && cy < -r {
        return;
    }
    let x0 = (cx - r).max(0) as usize;
    let y0 = (cy - r).max(0) as usize;
    let x1 = ((cx + r).max(0) as usize).min(bw - 1);
    let y1 = ((cy + r).max(0) as usize).min(bh - 1);
    let r2 = r * r;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            if dx * dx + dy * dy <= r2 {
                let i = (y * bw + x) * 4;
                buf[i] = c[2];
                buf[i + 1] = c[1];
                buf[i + 2] = c[0];
                buf[i + 3] = c[3];
            }
        }
    }
}

fn fill_quad(
    buf: &mut [u8],
    bw: usize,
    bh: usize,
    x0: f32,
    y0: f32,
    r0: f32,
    x1: f32,
    y1: f32,
    r1: f32,
    c: [u8; 4],
) {
    let ddx = x1 - x0;
    let ddy = y1 - y0;
    if !ddx.is_finite() || !ddy.is_finite() {
        return;
    }
    let len = (ddx * ddx + ddy * ddy).sqrt();
    if len < 0.5 {
        // Degenerate quad (points stacked): draw a circle at the midpoint instead
        fill_circle(
            buf,
            bw,
            bh,
            ((x0 + x1) * 0.5).round() as i32,
            ((y0 + y1) * 0.5).round() as i32,
            ((r0 + r1) * 0.5).round() as i32,
            c,
        );
        return;
    }
    let nx = -ddy / len;
    let ny = ddx / len;
    let ax = x0 + nx * r0;
    let ay = y0 + ny * r0;
    let bx = x0 - nx * r0;
    let by = y0 - ny * r0;
    let ccx = x1 + nx * r1;
    let ccy = y1 + ny * r1;
    let ddx2 = x1 - nx * r1;
    let ddy2 = y1 - ny * r1;
    let min_x = ax.min(bx).min(ccx).min(ddx2).max(0.0) as i32;
    let max_x = ax.max(bx).max(ccx).max(ddx2).min(bw as f32 - 1.0) as i32;
    let min_y = ay.min(by).min(ccy).min(ddy2).max(0.0) as i32;
    let max_y = ay.max(by).max(ccy).max(ddy2).min(bh as f32 - 1.0) as i32;
    fn sgn(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        (px - x2) * (y1 - y2) - (x1 - x2) * (py - y2)
    }
    fn in_tri(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let d1 = sgn(px, py, x0, y0, x1, y1);
        let d2 = sgn(px, py, x1, y1, x2, y2);
        let d3 = sgn(px, py, x2, y2, x0, y0);
        let neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
        let pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
        !(neg && pos)
    }
    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let pxf = px as f32 + 0.5;
            let pyf = py as f32 + 0.5;
            if in_tri(pxf, pyf, ax, ay, bx, by, ccx, ccy)
                || in_tri(pxf, pyf, bx, by, ccx, ccy, ddx2, ddy2)
            {
                let idx = (py as usize * bw + px as usize) * 4;
                buf[idx] = c[2];
                buf[idx + 1] = c[1];
                buf[idx + 2] = c[0];
                buf[idx + 3] = c[3];
            }
        }
    }
}

fn fill_quad_4pts(
    buf: &mut [u8],
    bw: usize,
    bh: usize,
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
    cx: f32,
    cy: f32,
    dx: f32,
    dy: f32,
    c: [u8; 4],
) {
    if bw == 0 || bh == 0 {
        return;
    }
    let min_x = ax.min(bx).min(cx).min(dx).max(0.0) as i32;
    let max_x = ax.max(bx).max(cx).max(dx).min(bw as f32 - 1.0) as i32;
    let min_y = ay.min(by).min(cy).min(dy).max(0.0) as i32;
    let max_y = ay.max(by).max(cy).max(dy).min(bh as f32 - 1.0) as i32;

    fn sgn(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        (px - x2) * (y1 - y2) - (x1 - x2) * (py - y2)
    }
    fn in_tri(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
        let d1 = sgn(px, py, x0, y0, x1, y1);
        let d2 = sgn(px, py, x1, y1, x2, y2);
        let d3 = sgn(px, py, x2, y2, x0, y0);
        let neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
        let pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
        !(neg && pos)
    }

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let pxf = px as f32 + 0.5;
            let pyf = py as f32 + 0.5;
            if in_tri(pxf, pyf, ax, ay, bx, by, cx, cy) || in_tri(pxf, pyf, bx, by, cx, cy, dx, dy)
            {
                let idx = (py as usize * bw + px as usize) * 4;
                buf[idx] = c[2];
                buf[idx + 1] = c[1];
                buf[idx + 2] = c[0];
                buf[idx + 3] = c[3];
            }
        }
    }
}

fn draw_tail(buf: &mut [u8], bw: usize, bh: usize, tail: &Tail, color: usize, scale: f32) {
    let n = tail.pts.len();
    if n < 2 || !tail.ready {
        return;
    }
    if !tail.pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()) {
        return;
    }
    // Colours sampled from the actual cat PNG assets
    let (body, bdr, stripe_out, stripe_fill): ([u8; 4], [u8; 4], Option<[u8; 4]>, Option<[u8; 4]>) =
        match color {
            0 => ([48, 48, 48, 255], [93, 93, 93, 255], None, None),
            1 => ([239, 239, 239, 255], [93, 93, 93, 255], None, None),
            2 => (
                [225, 158, 96, 255],
                [93, 93, 93, 255],
                Some([104, 70, 42, 255]),
                Some([168, 113, 67, 255]),
            ),
            _ => ([48, 48, 48, 255], [93, 93, 93, 255], None, None),
        };
    let br = TAIL_BASE_R * scale;
    let tr = TAIL_TIP_R * scale;
    let bw2 = TAIL_BORDER_W * scale;
    let s = scale;

    let get_radius = |i: usize| -> f32 {
        let t = i as f32 / (n - 1) as f32;
        br + (tr - br) * t
    };

    // 1. Draw outer border node circles
    for i in 0..n {
        let r = get_radius(i) + bw2;
        let px = (tail.pts[i].x * s).round() as i32;
        let py = (tail.pts[i].y * s).round() as i32;
        fill_circle(buf, bw, bh, px, py, r.round() as i32, bdr);
    }
    // 1b. Draw outer border quads between adjacent nodes
    for i in 0..n - 1 {
        let r0 = get_radius(i) + bw2;
        let r1 = get_radius(i + 1) + bw2;
        fill_quad(
            buf,
            bw,
            bh,
            tail.pts[i].x * s,
            tail.pts[i].y * s,
            r0,
            tail.pts[i + 1].x * s,
            tail.pts[i + 1].y * s,
            r1,
            bdr,
        );
    }

    // 2. Draw inner body node circles
    for i in 0..n {
        let r = get_radius(i);
        let px = (tail.pts[i].x * s).round() as i32;
        let py = (tail.pts[i].y * s).round() as i32;
        fill_circle(buf, bw, bh, px, py, r.round() as i32, body);
    }
    // 2b. Draw inner body quads between adjacent nodes
    for i in 0..n - 1 {
        let r0 = get_radius(i);
        let r1 = get_radius(i + 1);
        fill_quad(
            buf,
            bw,
            bh,
            tail.pts[i].x * s,
            tail.pts[i].y * s,
            r0,
            tail.pts[i + 1].x * s,
            tail.pts[i + 1].y * s,
            r1,
            body,
        );
    }

    // 3. Draw 3 stripes for orange cat matching the original asset
    if let (Some(so), Some(sf)) = (stripe_out, stripe_fill) {
        let stripe_indices = [
            (n as f32 * 0.25).round() as usize,
            (n as f32 * 0.50).round() as usize,
            (n as f32 * 0.75).round() as usize,
        ];
        for &idx in &stripe_indices {
            if idx == 0 || idx >= n - 1 {
                continue;
            }
            let i0 = idx - 1;
            let i1 = idx + 1;
            let x0 = tail.pts[i0].x * s;
            let y0 = tail.pts[i0].y * s;
            let x1 = tail.pts[i1].x * s;
            let y1 = tail.pts[i1].y * s;
            let dx = x1 - x0;
            let dy = y1 - y0;
            let len = (dx * dx + dy * dy).sqrt();
            if len > 0.001 {
                let ux = dx / len;
                let uy = dy / len;
                let nx = -uy;
                let ny = ux;
                let px = tail.pts[idx].x * s;
                let py = tail.pts[idx].y * s;
                let r = (get_radius(idx) - 2.0 * s).max(2.0);

                let w_out = 12.0 * s;
                let w_in = 6.0 * s;

                // Outer dark-brown stripe outline quad
                let c0x = px - nx * r - ux * w_out;
                let c0y = py - ny * r - uy * w_out;
                let c1x = px + nx * r - ux * w_out;
                let c1y = py + ny * r - uy * w_out;
                let c2x = px + nx * r + ux * w_out;
                let c2y = py + ny * r + uy * w_out;
                let c3x = px - nx * r + ux * w_out;
                let c3y = py - ny * r + uy * w_out;
                fill_quad_4pts(buf, bw, bh, c0x, c0y, c1x, c1y, c2x, c2y, c3x, c3y, so);

                // Inner amber stripe fill quad
                let i0x = px - nx * r - ux * w_in;
                let i0y = py - ny * r - uy * w_in;
                let i1x = px + nx * r - ux * w_in;
                let i1y = py + ny * r - uy * w_in;
                let i2x = px + nx * r + ux * w_in;
                let i2y = py + ny * r + uy * w_in;
                let i3x = px - nx * r + ux * w_in;
                let i3y = py - ny * r + uy * w_in;
                fill_quad_4pts(buf, bw, bh, i0x, i0y, i1x, i1y, i2x, i2y, i3x, i3y, sf);
            }
        }
    }
}

pub(crate) struct CatRenderer {
    pub base: Layer,
    pub closed: Layer,
    pub eyes: Layer,
    pub hl: Layer,
    pub annoyed: Option<Layer>,
    pub scarf: Option<Layer>,
    pub bell: Option<Layer>,
    pub tie: Option<Layer>,
    pub cos_scarf_cache: Vec<u8>,
    pub cos_scarf_idx: Option<usize>,
    pub cos_bell_cache: Vec<u8>,
    pub cos_bell_idx: Option<usize>,
    pub cos_tie_cache: Vec<u8>,
    pub cos_tie_idx: Option<usize>,
    pub buf: Vec<u8>,
    scratch: Vec<u8>,
    pub off_x: f32,
    pub off_y: f32,
    pub blink_start: Option<Instant>,
    pub blink_next: Instant,
    pub look_down_start: Option<Instant>,
    pub wander_t: f32,
    pub wander_skip: u32,
    pub cursor_last: POINT,
    pub cursor_still: f32,
    pub cached_rgba: [Vec<u8>; 3],
    pub cached_closed: [Vec<u8>; 3],
    pub cached_annoyed: [Vec<u8>; 3],
    pub cached_eyes: Vec<u8>,
    pub cached_hl: Vec<u8>,
    pub cached_w: u32,
    pub cached_h: u32,
    pub color: usize,
    pub wanted_scarf: Option<usize>,
    pub wanted_bell: Option<usize>,
    pub wanted_tie: Option<usize>,
    pub drag_active: bool,
    pub prev_drag: bool,
    pub fast_drag: bool,
    shake_time: f32,
    frame_time: Option<Instant>,
    pub spiral_start: Option<Instant>,
    pub spiral_dur: f32,
    pub spiral_angle: f32,
    pub tilt_off_x: f32,
    pub tilt_off_y: f32,
    pub tilt_scale: f32,
    pub tilt_layers: [Option<Layer>; 3],
    pub tilt_pupil: Option<Layer>,
    pub tilt_pupil_cache: Vec<u8>,
    pub tilt_scarf: Option<Layer>,
    pub tilt_bell: Option<Layer>,
    pub tilt_tie: Option<Layer>,
    pub tilt_scarf_cache: Vec<u8>,
    pub tilt_scarf_idx: Option<usize>,
    pub tilt_bell_cache: Vec<u8>,
    pub tilt_bell_idx: Option<usize>,
    pub tilt_tie_cache: Vec<u8>,
    pub tilt_tie_idx: Option<usize>,
    pub tilt_wanted_scarf: Option<usize>,
    pub tilt_wanted_bell: Option<usize>,
    pub tilt_wanted_tie: Option<usize>,
    pub sl_cache: Vec<u8>,
    pub sr_cache: Vec<u8>,
    pub spiral_left: Option<Vec<Layer>>,
    pub spiral_right: Option<Vec<Layer>>,
    pub spiral_scale: f32,
    pub tail: Tail,
    pub prev_pos_x: i32,
    pub prev_pos_y: i32,
    pub pos_initialized: bool,
    pub built_scale: f32,
    pub built_color: Option<usize>,
    pub built_cos_scale: f32,
    pub built_cos_tuple: (Option<usize>, Option<usize>, Option<usize>),
}

impl CatRenderer {
    pub(crate) fn new() -> Self {
        CatRenderer {
            base: Layer::new(0, 0),
            closed: Layer::new(0, 0),
            eyes: Layer::new(0, 0),
            hl: Layer::new(0, 0),
            annoyed: None,
            scarf: None,
            bell: None,
            tie: None,
            cos_scarf_cache: Vec::new(),
            cos_scarf_idx: None,
            cos_bell_cache: Vec::new(),
            cos_bell_idx: None,
            cos_tie_cache: Vec::new(),
            cos_tie_idx: None,
            buf: Vec::new(),
            scratch: Vec::new(),
            off_x: 0.0,
            off_y: 0.0,
            blink_start: None,
            blink_next: Instant::now(),
            look_down_start: None,
            wander_t: 0.0,
            wander_skip: 0,
            cursor_last: POINT { x: 0, y: 0 },
            cursor_still: 0.0,
            cached_rgba: [Vec::new(), Vec::new(), Vec::new()],
            cached_closed: [Vec::new(), Vec::new(), Vec::new()],
            cached_annoyed: [Vec::new(), Vec::new(), Vec::new()],
            cached_eyes: Vec::new(),
            cached_hl: Vec::new(),
            cached_w: 0,
            cached_h: 0,
            color: 0,
            wanted_scarf: None,
            wanted_bell: None,
            wanted_tie: None,
            drag_active: false,
            prev_drag: false,
            fast_drag: false,
            shake_time: 0.0,
            frame_time: None,
            spiral_start: None,
            spiral_dur: 2.0,
            spiral_angle: 0.0,
            tilt_off_x: 0.0,
            tilt_off_y: 0.0,
            tilt_scale: -1.0,
            tilt_layers: [None, None, None],
            tilt_pupil: None,
            tilt_pupil_cache: Vec::new(),
            tilt_scarf: None,
            tilt_bell: None,
            tilt_tie: None,
            tilt_scarf_cache: Vec::new(),
            tilt_scarf_idx: None,
            tilt_bell_cache: Vec::new(),
            tilt_bell_idx: None,
            tilt_tie_cache: Vec::new(),
            tilt_tie_idx: None,
            tilt_wanted_scarf: None,
            tilt_wanted_bell: None,
            tilt_wanted_tie: None,
            sl_cache: Vec::new(),
            sr_cache: Vec::new(),
            spiral_left: None,
            spiral_right: None,
            spiral_scale: -1.0,
            tail: Tail::new(),
            prev_pos_x: 0,
            prev_pos_y: 0,
            pos_initialized: false,
            built_scale: -1.0,
            built_color: None,
            built_cos_scale: -1.0,
            built_cos_tuple: (None, None, None),
        }
    }

    pub(crate) fn rebuild_layers(&mut self, color: usize, w: i32, h: i32, scale: f32) {
        let color = color.min(2);
        self.color = color;
        if self.built_color == Some(color)
            && (self.built_scale - scale).abs() < 0.001
            && !self.base.data.is_empty()
        {
            return;
        }
        self.built_color = Some(color);
        self.built_scale = scale;

        let socket_bytes = [BLACK_SOCKET, WHITE_SOCKET, ORANGE_SOCKET][color];
        let closed_bytes = [BLACK_CLOSED, WHITE_CLOSED, ORANGE_CLOSED][color];
        let annoyed_bytes = [BLACK_ANNOYED, WHITE_ANNOYED, ORANGE_ANNOYED][color];

        let crop = (CONTENT_X, CONTENT_Y, CONTENT_W, CONTENT_H);

        if self.cached_eyes.is_empty() {
            let (eyes_rgba, w, h) = decode_and_crop(EYES, crop);
            self.cached_eyes = eyes_rgba;
            let (hl_rgba, _, _) = decode_and_crop(HIGHLIGHT, crop);
            self.cached_hl = hl_rgba;
            self.cached_w = w;
            self.cached_h = h;
        }
        let cw = self.cached_w;
        let ch = self.cached_h;

        if self.cached_rgba[color].is_empty() {
            let (rgba, _, _) = decode_and_crop(socket_bytes, crop);
            self.cached_rgba[color] = rgba;
        }
        if self.cached_closed[color].is_empty() {
            self.cached_closed[color] = decode_and_crop(closed_bytes, crop).0;
        }
        if self.cached_annoyed[color].is_empty() {
            self.cached_annoyed[color] = decode_and_crop(annoyed_bytes, crop).0;
        }

        self.base = load_layer_cached(&self.cached_rgba[color], cw, ch, scale);
        self.closed = load_layer_cached(&self.cached_closed[color], cw, ch, scale);
        self.annoyed = Some(load_layer_cached(
            &self.cached_annoyed[color],
            cw,
            ch,
            scale,
        ));

        self.eyes = load_layer_cached(&self.cached_eyes, cw, ch, scale);
        self.hl = load_layer_cached(&self.cached_hl, cw, ch, scale);

        let _ = w;
        let _ = h;
    }

    pub(crate) fn rebuild_cosmetics(
        &mut self,
        bell: Option<usize>,
        scarf: Option<usize>,
        tie: Option<usize>,
        scale: f32,
    ) {
        let wanted = (bell, scarf, tie);
        if self.built_cos_tuple == wanted && (self.built_cos_scale - scale).abs() < 0.001 {
            return;
        }
        self.built_cos_tuple = wanted;
        self.built_cos_scale = scale;

        let crop = (CONTENT_X, CONTENT_Y, CONTENT_W, CONTENT_H);
        self.wanted_scarf = scarf;
        self.wanted_bell = bell;
        self.wanted_tie = tie;
        self.scarf = load_cos_layer(
            &mut self.cos_scarf_cache,
            &mut self.cos_scarf_idx,
            scarf,
            super::cosmetics::SCARF_ITEMS,
            crop,
            scale,
        );
        self.bell = load_cos_layer(
            &mut self.cos_bell_cache,
            &mut self.cos_bell_idx,
            bell,
            super::cosmetics::BELL_ITEMS,
            crop,
            scale,
        );
        self.tie = load_cos_layer(
            &mut self.cos_tie_cache,
            &mut self.cos_tie_idx,
            tie,
            super::cosmetics::TIE_ITEMS,
            crop,
            scale,
        );
    }

    pub(crate) fn set_look_down(&mut self) {
        self.look_down_start = Some(Instant::now());
    }

    pub(crate) fn compose_annoyed(&mut self, w: usize, h: usize) -> bool {
        let Some(annoyed) = &self.annoyed else {
            return false;
        };
        self.buf.resize(w * h * 4, 0);
        self.buf.fill(0);
        blit(
            &mut self.buf,
            w,
            h,
            &annoyed.data,
            annoyed.w,
            annoyed.h,
            0,
            0,
        );
        for layer in [&self.scarf, &self.bell, &self.tie].into_iter().flatten() {
            blit(&mut self.buf, w, h, &layer.data, layer.w, layer.h, 0, 0);
        }
        true
    }

    pub(crate) fn ensure_tilted(&mut self, scale: f32) {
        let crop = (TILT_X, TILT_Y, TILT_W, TILT_H);
        if self.tilt_scale != scale {
            self.tilt_layers = [None, None, None];
            self.tilt_pupil = None;
            self.tilt_scarf = None;
            self.tilt_bell = None;
            self.tilt_tie = None;
            self.tilt_scale = scale;
        }
        if self.tilt_layers[self.color].is_none() {
            let (rgba, w, h) = decode_and_crop(TILTED_SOCKETS[self.color], crop);
            self.tilt_layers[self.color] = Some(load_layer_cached(&rgba, w, h, scale));
        }
        if self.tilt_pupil.is_none() {
            if self.tilt_pupil_cache.is_empty() {
                self.tilt_pupil_cache = decode_and_crop(TILTED_PUPIL, crop).0;
            }
            self.tilt_pupil = Some(load_layer_cached(
                &self.tilt_pupil_cache,
                TILT_W as u32,
                TILT_H as u32,
                scale,
            ));
        }
        let rebuild_cos = self.tilt_wanted_scarf != self.wanted_scarf
            || self.tilt_wanted_bell != self.wanted_bell
            || self.tilt_wanted_tie != self.wanted_tie
            || (self.wanted_scarf.is_some() && self.tilt_scarf.is_none())
            || (self.wanted_bell.is_some() && self.tilt_bell.is_none())
            || (self.wanted_tie.is_some() && self.tilt_tie.is_none());
        if rebuild_cos {
            self.tilt_wanted_scarf = self.wanted_scarf;
            self.tilt_wanted_bell = self.wanted_bell;
            self.tilt_wanted_tie = self.wanted_tie;
            self.tilt_scarf = load_cos_layer(
                &mut self.tilt_scarf_cache,
                &mut self.tilt_scarf_idx,
                self.wanted_scarf,
                TILTED_SCARF_ITEMS,
                crop,
                scale,
            );
            self.tilt_bell = load_cos_layer(
                &mut self.tilt_bell_cache,
                &mut self.tilt_bell_idx,
                self.wanted_bell,
                TILTED_BELL_ITEMS,
                crop,
                scale,
            );
            self.tilt_tie = load_cos_layer(
                &mut self.tilt_tie_cache,
                &mut self.tilt_tie_idx,
                self.wanted_tie,
                TILTED_TIE_ITEMS,
                crop,
                scale,
            );
        }
    }

    pub(crate) fn build_spiral_frames(&mut self, scale: f32) {
        let crop = (CONTENT_X, CONTENT_Y, CONTENT_W, CONTENT_H);
        if self.sl_cache.is_empty() {
            self.sl_cache = decode_and_crop(SPIRAL_LEFT, crop).0;
        }
        if self.sr_cache.is_empty() {
            self.sr_cache = decode_and_crop(SPIRAL_RIGHT, crop).0;
        }
        let l = load_layer_cached(&self.sl_cache, self.cached_w, self.cached_h, scale);
        let r = load_layer_cached(&self.sr_cache, self.cached_w, self.cached_h, scale);
        // Socket centres measured from eyes.png dark pixel mass: left eye (590,508)-(893,957)
        // centre ~(746,739); right eye (1133,489)-(1450,945) centre ~(1295,725). NOTE:
        // spiralpupilleft.png sits on the RIGHT socket, spiralpupilright.png on the LEFT.
        // Rotating around the socket centre keeps the spiral spinning in place inside it.
        let (lcx, lcy) = (
            (1295.0_f32 - CONTENT_X as f32) * scale,
            (725.0_f32 - CONTENT_Y as f32) * scale,
        );
        let (rcx, rcy) = (
            (746.0_f32 - CONTENT_X as f32) * scale,
            (739.0_f32 - CONTENT_Y as f32) * scale,
        );
        let mut lv = Vec::with_capacity(12);
        let mut rv = Vec::with_capacity(12);
        for k in 0..12usize {
            lv.push(rotate_layer_nearest(&l, k as f32 * 30.0, lcx, lcy));
            rv.push(rotate_layer_nearest(&r, k as f32 * 30.0, rcx, rcy));
        }
        self.spiral_left = Some(lv);
        self.spiral_right = Some(rv);
        self.spiral_scale = scale;
    }

    pub(crate) fn rebuild_dib(
        &mut self,
        hdc_mem: *mut c_void,
        w: i32,
        h: i32,
    ) -> (*mut c_void, *mut c_void) {
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 1],
        };
        let mut bits: *mut c_void = null_mut();
        let hbmp = unsafe {
            CreateDIBSection(
                hdc_mem,
                &bmi as *const _,
                DIB_RGB_COLORS,
                &mut bits as *mut _ as *mut *mut c_void,
                null_mut(),
                0,
            )
        };
        (hbmp, bits)
    }

    // Called by the main window's timer every FRAME_MS before render.
    // Returns true when this tick needs a full composite + UploadLayeredWindow;
    // false lets the timer skip the frame entirely (idle cat = near-zero CPU).
    pub(crate) fn tick(&mut self, mouse_moved: bool) -> bool {
        let now = Instant::now();
        if mouse_moved {
            self.cursor_still = 0.0;
        } else {
            self.cursor_still += super::FRAME_MS as f32 / 1000.0;
        }
        if self.blink_start.is_some() {
            return true;
        }
        if now >= self.blink_next - std::time::Duration::from_millis(250) {
            return true;
        }
        if self.look_down_start.is_some() {
            return true;
        }
        if self.drag_active || self.spiral_start.is_some() {
            return true;
        }
        if self.cursor_still > 1.5 {
            // Idle eye-wandering: throttle to about 8 fps.
            self.wander_skip += 1;
            if self.wander_skip < 8 {
                return false;
            }
            self.wander_skip = 0;
        }
        true
    }

    pub(crate) fn render(
        &mut self,
        pos_x: i32,
        pos_y: i32,
        w: i32,
        h: i32,
        scale: f32,
        hdc_screen: *mut c_void,
        hdc_mem: *mut c_void,
        bits: *mut c_void,
        hwnd: HWND,
    ) {
        let mut cursor = POINT { x: 0, y: 0 };
        unsafe {
            GetCursorPos(&mut cursor);
        }
        self.compose_frame(pos_x, pos_y, w, h, scale, cursor, Instant::now());
        if self.buf.len() != (w * h * 4) as usize || bits.is_null() {
            return;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(self.buf.as_ptr(), bits as *mut u8, self.buf.len());
            let source = POINT { x: 0, y: 0 };
            let size = SIZE { cx: w, cy: h };
            let blend = BLENDFUNCTION {
                BlendOp: 0,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: 1,
            };
            UpdateLayeredWindow(
                hwnd,
                hdc_screen,
                null(),
                &size,
                hdc_mem,
                &source,
                0,
                &blend,
                ULW_ALPHA,
            );
        }
    }

    // Frame composition takes explicit time/cursor input so animation transitions
    // can be checked deterministically without manipulating the user's desktop.
    fn compose_frame(
        &mut self,
        pos_x: i32,
        pos_y: i32,
        w: i32,
        h: i32,
        scale: f32,
        pt: POINT,
        now: Instant,
    ) {
        let bw = w as usize;
        let bh = h as usize;
        let cw = self.base.w;
        let ch = self.base.h;

        if cw == 0 || ch == 0 {
            return;
        }

        // Buffer must match the window, which can be the tilted aspect while dragging
        if self.buf.len() != bw * bh * 4 {
            self.buf = vec![0u8; bw * bh * 4];
        }

        let cat_center_x = pos_x as f32 + cw as f32 / 2.0;
        let cat_center_y = pos_y as f32 + ch as f32 / 2.0;

        let dx = pt.x as f32 - cat_center_x;
        let dy = pt.y as f32 - cat_center_y;

        let dist = (dx * dx + dy * dy).sqrt();

        let mut target_x = 0.0;
        let mut target_y = 0.0;
        if dist > DEADZONE {
            target_x = dx / dist * MAX_OFF_X;
            target_y = dy / dist * MAX_OFF_Y;
            // Keep the pupil inside the (circular) socket: clamp the offset magnitude
            let m = (target_x * target_x + target_y * target_y).sqrt();
            if m > MAX_OFF_Y {
                let k = MAX_OFF_Y / m;
                target_x *= k;
                target_y *= k;
            }
        }

        let dt = self
            .frame_time
            .map(|last| now.saturating_duration_since(last).as_secs_f32())
            .unwrap_or(super::FRAME_MS as f32 / 1000.0)
            .clamp(0.001, 0.25);
        self.frame_time = Some(now);

        let win_dx = if self.pos_initialized {
            pos_x - self.prev_pos_x
        } else {
            0
        };
        let win_dy = if self.pos_initialized {
            pos_y - self.prev_pos_y
        } else {
            0
        };
        self.prev_pos_x = pos_x;
        self.prev_pos_y = pos_y;
        self.pos_initialized = true;

        let mut dx_pos = pt.x - self.cursor_last.x;
        let mut dy_pos = pt.y - self.cursor_last.y;
        self.cursor_last = pt;

        // Drag & tilt state: show tilted cat while held, spiral eyes after a fast drag
        if self.drag_active != self.prev_drag {
            if self.drag_active {
                self.fast_drag = false;
                self.shake_time = 0.0;
                self.spiral_start = None;
                dx_pos = 0;
                dy_pos = 0;
                self.tilt_off_x = 0.0;
                self.tilt_off_y = 0.0;
                self.tail.reset();
            } else {
                if self.fast_drag {
                    self.spiral_start = Some(now);
                    self.spiral_dur = if rand::rng().random::<bool>() {
                        2.0
                    } else {
                        3.0
                    };
                    self.spiral_angle = 0.0;
                }
                self.fast_drag = false;
                self.shake_time = 0.0;
                self.tail.reset();
            }
            self.prev_drag = self.drag_active;
        }

        let dpx = dx_pos as f32;
        let dpy = dy_pos as f32;
        let mut tilt_active = false;
        if self.drag_active {
            let speed = (dpx * dpx + dpy * dpy).sqrt();
            // Use elapsed time rather than timer tick count: delayed frames must
            // not turn an ordinary slow drag into a dizzy reaction.
            if speed / dt > 1100.0 {
                self.shake_time += dt;
                if self.shake_time >= 0.048 {
                    self.fast_drag = true;
                }
            } else {
                self.shake_time = 0.0;
            }
            let m = speed.max(1.0);
            let amp = 7.0 * scale;
            let tx = -(dpx / m) * amp;
            let ty = -(dpy / m) * amp;
            self.tilt_off_x += (tx - self.tilt_off_x) * 0.3;
            self.tilt_off_y += (ty - self.tilt_off_y) * 0.3;
            tilt_active = true;
        } else {
            self.tilt_off_x *= 0.8;
            self.tilt_off_y *= 0.8;
        }

        let spiral_active = if let Some(t0) = self.spiral_start {
            if now.duration_since(t0).as_secs_f32() >= self.spiral_dur + 0.35 {
                self.spiral_start = None;
                false
            } else {
                if now.duration_since(t0).as_secs_f32() < self.spiral_dur {
                    self.spiral_angle += 130.0 * dt;
                }
                true
            }
        } else {
            false
        };

        if spiral_active {
            // During the spiral the eyes stay planted in their sockets - no cursor tracking
            self.off_x *= 0.86;
            self.off_y *= 0.86;
        } else if self.cursor_still > 1.5 {
            self.wander_t += dt;
            let wander_amp_x = MAX_OFF_X * 0.15;
            let wander_amp_y = MAX_OFF_Y * 0.1;
            let wander_x = (self.wander_t * 0.7).sin() * wander_amp_x;
            let wander_y = (self.wander_t * 1.1).sin() * wander_amp_y;
            self.off_x += (wander_x - self.off_x) * 0.02;
            self.off_y += (wander_y - self.off_y) * 0.02;
        } else {
            self.wander_t = 0.0;
            self.off_x += (target_x - self.off_x) * 0.16;
            self.off_y += (target_y - self.off_y) * 0.16;
        }

        let blink_dur = 0.27f32;
        let fade_in = 0.11f32;
        let hold = 0.05f32;
        let fade_out = 0.11f32;

        if self.blink_start.is_none() && now >= self.blink_next {
            self.blink_start = Some(now);
            let next_secs = 3.0 + rand::rng().random::<f32>() * 4.5;
            self.blink_next = now + std::time::Duration::from_secs_f32(next_secs);
        }

        let mut blink_alpha: f32 = 0.0;
        if let Some(start) = self.blink_start {
            let elapsed = now.duration_since(start).as_secs_f32();
            if elapsed < fade_in {
                blink_alpha = elapsed / fade_in;
            } else if elapsed < fade_in + hold {
                blink_alpha = 1.0;
            } else if elapsed < blink_dur {
                blink_alpha = 1.0 - (elapsed - fade_in - hold) / fade_out;
            } else {
                blink_alpha = 0.0;
                self.blink_start = None;
            }
        }

        let mut eye_offset_x = (self.off_x * scale) as i32;
        let mut eye_offset_y = (self.off_y * scale) as i32;

        // When a cosmetic is applied, eyes glance down for ~1s then revert
        if let Some(start) = self.look_down_start {
            let elapsed = now.duration_since(start).as_secs_f32();
            if elapsed < 1.0 {
                let t = (elapsed / 1.0).min(1.0);
                let ease = t * t * (3.0 - 2.0 * t);
                let down = (MAX_OFF_Y * 0.5 * ease * scale) as i32;
                eye_offset_x = 0;
                eye_offset_y = down;
            } else {
                self.look_down_start = None;
            }
        }

        self.buf.iter_mut().for_each(|b| *b = 0);

        let base_x = (bw as i32 - cw as i32) / 2;
        let base_y = (bh as i32 - ch as i32) / 2;

        if tilt_active {
            // --- Dragged: tilted cat, pupils drift slightly opposite to motion ---
            // The window is resized to the tilted canvas, so everything blits from (0,0)
            self.ensure_tilted(scale);
            let tilt_ready = self.tilt_layers[self.color]
                .as_ref()
                .map(|t| {
                    t.w.abs_diff(bw) <= 2
                        && t.h <= bh
                        && ((TILT_CANVAS_H as f32 * scale).round() as usize).abs_diff(bh) <= 2
                })
                .unwrap_or(false);
            if tilt_ready {
                // Tail hangs from the butt; physics in canvas coords, drawn behind the body
                let sc = scale.max(0.01);
                let ax = TAIL_AX + self.tilt_off_x / sc;
                let ay = TAIL_AY + self.tilt_off_y / sc;
                let win_dx_l = win_dx as f32 / sc;
                let win_dy_l = win_dy as f32 / sc;
                self.tail
                    .update(ax, ay, win_dx_l, win_dy_l, dt.min(1.0 / 30.0));
                draw_tail(&mut self.buf, bw, bh, &self.tail, self.color, scale);
                if let Some(t) = &self.tilt_layers[self.color] {
                    blit(&mut self.buf, bw, bh, &t.data, t.w, t.h, 0, 0);
                }
                if let Some(s) = &self.tilt_scarf {
                    blit(&mut self.buf, bw, bh, &s.data, s.w, s.h, 0, 0);
                }
                if let Some(b) = &self.tilt_bell {
                    blit(&mut self.buf, bw, bh, &b.data, b.w, b.h, 0, 0);
                }
                if let Some(t) = &self.tilt_tie {
                    blit(&mut self.buf, bw, bh, &t.data, t.w, t.h, 0, 0);
                }
                if let Some(p) = &self.tilt_pupil {
                    blit(
                        &mut self.buf,
                        bw,
                        bh,
                        &p.data,
                        p.w,
                        p.h,
                        self.tilt_off_x.round() as i32,
                        self.tilt_off_y.round() as i32,
                    );
                }
            }
        } else {
            // --- Normal pose ---
            blit(
                &mut self.buf,
                bw,
                bh,
                &self.base.data,
                cw,
                ch,
                base_x,
                base_y,
            );

            // Cosmetics (scarf first, then bell or tie on top)
            if let Some(s) = &self.scarf {
                blit(&mut self.buf, bw, bh, &s.data, s.w, s.h, base_x, base_y);
            }
            if let Some(b) = &self.bell {
                blit(&mut self.buf, bw, bh, &b.data, b.w, b.h, base_x, base_y);
            }
            if let Some(t) = &self.tie {
                blit(&mut self.buf, bw, bh, &t.data, t.w, t.h, base_x, base_y);
            }

            let eyes_x = base_x + eye_offset_x;
            let eyes_y = base_y + eye_offset_y;
            blit(
                &mut self.buf,
                bw,
                bh,
                &self.eyes.data,
                self.eyes.w,
                self.eyes.h,
                eyes_x,
                eyes_y,
            );

            blit(
                &mut self.buf,
                bw,
                bh,
                &self.hl.data,
                self.hl.w,
                self.hl.h,
                eyes_x,
                eyes_y,
            );

            if blink_alpha > 0.01 && !spiral_active {
                let closed_layer = &self.closed;
                let mut temp = std::mem::take(&mut self.scratch);
                temp.resize(self.buf.len(), 0);
                temp.copy_from_slice(&self.buf);
                blit(
                    &mut temp,
                    bw,
                    bh,
                    &closed_layer.data,
                    closed_layer.w,
                    closed_layer.h,
                    base_x,
                    base_y,
                );
                // Keep cosmetics visible during the blink (closed layer covers them)
                if let Some(s) = &self.scarf {
                    blit(&mut temp, bw, bh, &s.data, s.w, s.h, base_x, base_y);
                }
                if let Some(b) = &self.bell {
                    blit(&mut temp, bw, bh, &b.data, b.w, b.h, base_x, base_y);
                }
                if let Some(t) = &self.tie {
                    blit(&mut temp, bw, bh, &t.data, t.w, t.h, base_x, base_y);
                }
                crossfade(&mut self.buf, &temp, blink_alpha);
                self.scratch = temp;
            }

            // --- Spiral eyes after fast drag, lingering then fading back to real eyes ---
            if spiral_active {
                if self.spiral_left.is_none() || self.spiral_scale != scale {
                    self.build_spiral_frames(scale);
                }
                let now_sp = now.duration_since(self.spiral_start.unwrap()).as_secs_f32();
                let fade_out = if now_sp >= self.spiral_dur {
                    ((now_sp - self.spiral_dur) / 0.35).min(1.0)
                } else {
                    0.0
                };
                self.scratch.resize(self.buf.len(), 0);
                self.scratch.copy_from_slice(&self.buf);
                blit(
                    &mut self.buf,
                    bw,
                    bh,
                    &self.base.data,
                    cw,
                    ch,
                    base_x,
                    base_y,
                );
                if let Some(s) = &self.scarf {
                    blit(&mut self.buf, bw, bh, &s.data, s.w, s.h, base_x, base_y);
                }
                if let Some(b) = &self.bell {
                    blit(&mut self.buf, bw, bh, &b.data, b.w, b.h, base_x, base_y);
                }
                if let Some(t) = &self.tie {
                    blit(&mut self.buf, bw, bh, &t.data, t.w, t.h, base_x, base_y);
                }
                if let (Some(lv), Some(rv)) = (&self.spiral_left, &self.spiral_right) {
                    let idx = (((self.spiral_angle / 30.0) as usize) % 12).min(11);
                    if let (Some(l), Some(r)) = (lv.get(idx), rv.get(idx)) {
                        // Canvas-aligned like the real eyes: the spiral layers match the
                        // CONTENT crop, and each already rotates around its own pupil centre
                        blit(
                            &mut self.buf,
                            bw,
                            bh,
                            &l.data,
                            l.w,
                            l.h,
                            base_x + eye_offset_x,
                            base_y + eye_offset_y,
                        );
                        blit(
                            &mut self.buf,
                            bw,
                            bh,
                            &r.data,
                            r.w,
                            r.h,
                            base_x + eye_offset_x,
                            base_y + eye_offset_y,
                        );
                    }
                }
                crossfade(&mut self.buf, &self.scratch, fade_out);
            }
        }
    }
}

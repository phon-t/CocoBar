use image::imageops::FilterType;
use image::{load_from_memory_with_format, ImageFormat};
use rand::RngExt;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::time::Instant;
use windows_sys::Win32::Foundation::{HWND, POINT, SIZE};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
    CreateDIBSection, DIB_RGB_COLORS, RGBQUAD,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
    UpdateLayeredWindow, ULW_ALPHA,
};

pub(crate) const CONTENT_X: i32 = 364;
pub(crate) const CONTENT_Y: i32 = 148;
pub(crate) const CONTENT_W: i32 = 1192;
pub(crate) const CONTENT_H: i32 = 1748;
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

#[derive(Clone)]
pub(crate) struct Layer {
    pub w: usize,
    pub h: usize,
    pub data: Vec<u8>,
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
    cache_w: &mut u32,
    cache_h: &mut u32,
    fallback_w: u32,
    fallback_h: u32,
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
        *cache_w = w;
        *cache_h = h;
    }
    let cw = if *cache_w > 0 { *cache_w } else { fallback_w };
    let ch = if *cache_h > 0 { *cache_h } else { fallback_h };
    Some(load_layer_cached(cache, cw, ch, scale))
}

pub(crate) fn decode_and_crop(
    bytes: &[u8],
    crop: (i32, i32, i32, i32),
) -> (Vec<u8>, u32, u32) {
    let img = load_from_memory_with_format(bytes, ImageFormat::Png)
        .expect("png decode")
        .to_rgba8();
    let cropped = image::imageops::crop(
        &mut img.clone(),
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
    let mut img = image::RgbaImage::from_raw(cached_w, cached_h, cached_rgba.to_vec()).unwrap();
    let resized = image::imageops::resize(&mut img, dw, dh, FilterType::Triangle);
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
    pub cos_cache_w: u32,
    pub cos_cache_h: u32,
    pub buf: Vec<u8>,
    pub off_x: f32,
    pub off_y: f32,
    pub blink_start: Option<Instant>,
    pub blink_next: Instant,
    pub look_down_start: Option<Instant>,
    pub wander_t: f32,
    pub cursor_last: POINT,
    pub cursor_still: f32,
    pub cached_rgba: [Vec<u8>; 3],
    pub cached_closed: [Vec<u8>; 3],
    pub cached_annoyed: [Vec<u8>; 3],
    pub cached_eyes: Vec<u8>,
    pub cached_hl: Vec<u8>,
    pub cached_w: u32,
    pub cached_h: u32,
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
            cos_cache_w: 0,
            cos_cache_h: 0,
            buf: Vec::new(),
            off_x: 0.0,
            off_y: 0.0,
            blink_start: None,
            blink_next: Instant::now(),
            look_down_start: None,
            wander_t: 0.0,
            cursor_last: POINT { x: 0, y: 0 },
            cursor_still: 0.0,
            cached_rgba: [Vec::new(), Vec::new(), Vec::new()],
            cached_closed: [Vec::new(), Vec::new(), Vec::new()],
            cached_annoyed: [Vec::new(), Vec::new(), Vec::new()],
            cached_eyes: Vec::new(),
            cached_hl: Vec::new(),
            cached_w: 0,
            cached_h: 0,
        }
    }

    pub(crate) fn rebuild_layers(&mut self, color: usize, w: i32, h: i32, scale: f32) {
        let color = color.min(2);
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
        self.annoyed = Some(load_layer_cached(&self.cached_annoyed[color], cw, ch, scale));

        self.eyes = load_layer_cached(&self.cached_eyes, cw, ch, scale);
        self.hl = load_layer_cached(&self.cached_hl, cw, ch, scale);

        let _ = w;
        let _ = h;
        self.buf = vec![0u8; self.base.w * self.base.h * 4];
    }

    pub(crate) fn rebuild_cosmetics(
        &mut self,
        bell: Option<usize>,
        scarf: Option<usize>,
        tie: Option<usize>,
        scale: f32,
    ) {
        let crop = (CONTENT_X, CONTENT_Y, CONTENT_W, CONTENT_H);
        self.scarf = load_cos_layer(
            &mut self.cos_scarf_cache,
            &mut self.cos_scarf_idx,
            &mut self.cos_cache_w,
            &mut self.cos_cache_h,
            self.cached_w,
            self.cached_h,
            scarf,
            super::cosmetics::SCARF_ITEMS,
            crop,
            scale,
        );
        self.bell = load_cos_layer(
            &mut self.cos_bell_cache,
            &mut self.cos_bell_idx,
            &mut self.cos_cache_w,
            &mut self.cos_cache_h,
            self.cached_w,
            self.cached_h,
            bell,
            super::cosmetics::BELL_ITEMS,
            crop,
            scale,
        );
        self.tie = load_cos_layer(
            &mut self.cos_tie_cache,
            &mut self.cos_tie_idx,
            &mut self.cos_cache_w,
            &mut self.cos_cache_h,
            self.cached_w,
            self.cached_h,
            tie,
            super::cosmetics::TIE_ITEMS,
            crop,
            scale,
        );
    }

    pub(crate) fn set_look_down(&mut self) {
        self.look_down_start = Some(Instant::now());
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
        let bw = w as usize;
        let bh = h as usize;
        let cw = self.base.w;
        let ch = self.base.h;

        if cw == 0 || ch == 0 {
            return;
        }

        let mut pt = POINT { x: 0, y: 0 };
        unsafe { GetCursorPos(&mut pt) };

        let _screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let _screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };

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

        let now = Instant::now();
        let dt = super::FRAME_MS as f32 / 1000.0;

        let dx_pos = pt.x - self.cursor_last.x;
        let dy_pos = pt.y - self.cursor_last.y;
        if dx_pos.abs() > 2 || dy_pos.abs() > 2 {
            self.cursor_still = 0.0;
        } else {
            self.cursor_still += dt;
        }
        self.cursor_last = pt;

        if self.cursor_still > 1.5 {
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

        self.buf.iter_mut().for_each(|b| *b = 0);

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

        let base_x = ((cw as i32 - bw as i32) / 2) as i32;
        let base_y = ((ch as i32 - bh as i32) / 2) as i32;

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

        if blink_alpha > 0.01 {
            let closed_layer = &self.closed;
            let mut temp = vec![0u8; self.buf.len()];
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
        }

        if !bits.is_null() && !self.buf.is_empty() {
            unsafe {
                let dst = std::slice::from_raw_parts_mut(bits as *mut u8, bw * bh * 4);
                let src_bytes = self.buf.as_ptr();
                let src_len = self.buf.len();
                if src_len >= bw * bh * 4 {
                    std::ptr::copy_nonoverlapping(src_bytes, dst.as_mut_ptr(), bw * bh * 4);
                }
            }
        }

        let pt_src = POINT { x: 0, y: 0 };
        let size = SIZE { cx: bw as i32, cy: bh as i32 };
        let blend = BLENDFUNCTION {
            BlendOp: 0,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: 1,
        };

        unsafe {
            UpdateLayeredWindow(
                hwnd,
                hdc_screen,
                null(),
                &size,
                hdc_mem,
                &pt_src,
                0,
                &blend,
                ULW_ALPHA,
            );
        }
    }
}

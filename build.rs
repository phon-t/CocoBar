use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let ico_path = Path::new(&out_dir).join("app_icon.ico");

    generate_ico(&ico_path);

    let mut res = winres::WindowsResource::new();
    res.set_icon(ico_path.to_str().unwrap());
    res.set("FileDescription", "cocoBar - Desktop Cat Companion");
    res.set("ProductName", "cocoBar");
    res.set("ProductVersion", "0.1.0");
    res.compile().unwrap();
}

fn generate_ico(out: &Path) {
    let sizes: &[u32] = &[16, 32, 48, 256];
    let mut entries: Vec<(u32, Vec<u8>)> = Vec::new();

    for &sz in sizes {
        let img = image::load_from_memory(include_bytes!("assets/blackcatfull.png"))
            .expect("png decode")
            .resize(sz, sz, image::imageops::FilterType::Lanczos3)
            .to_rgba8();
        let raw = img.into_raw();
        entries.push((sz, raw));
    }

    let mut f = fs::File::create(out).unwrap();

    let num_images = entries.len() as u16;
    let dir_reserved = 0u16;
    let dir_type = 1u16;
    let dir_count = num_images;

    let header_size = 6u32;
    let entry_size = 16u32;
    let entries_size = entry_size * dir_count as u32;
    let data_offset = header_size + entries_size;

    f.write_all(&dir_reserved.to_le_bytes()).unwrap();
    f.write_all(&dir_type.to_le_bytes()).unwrap();
    f.write_all(&dir_count.to_le_bytes()).unwrap();

    let mut current_offset = data_offset;
    let mut image_data_blocks: Vec<Vec<u8>> = Vec::new();

    for (sz, raw) in &entries {
        let w = *sz as u16;
        let h = *sz as u16;
        let _bpp = 32u16;
        let _reserved = 0u16;
        let color_planes = 1u16;
        let bytes_per_pixel = 4u16;

        let bmp_header_size = 40u32;
        let bmp_width = *sz as i32;
        let bmp_height = *sz as i32 * 2;
        let planes = 1u16;
        let bit_count = 32u16;
        let compression = 0u32;
        let image_size = (*sz * *sz * 4) as u32;
        let pixels_per_meter = 0u32;
        let colors_used = 0u32;
        let colors_important = 0u32;

        let mut bmp = Vec::new();
        bmp.extend_from_slice(&bmp_header_size.to_le_bytes());
        bmp.extend_from_slice(&bmp_width.to_le_bytes());
        bmp.extend_from_slice(&bmp_height.to_le_bytes());
        bmp.extend_from_slice(&planes.to_le_bytes());
        bmp.extend_from_slice(&bit_count.to_le_bytes());
        bmp.extend_from_slice(&compression.to_le_bytes());
        bmp.extend_from_slice(&image_size.to_le_bytes());
        bmp.extend_from_slice(&pixels_per_meter.to_le_bytes());
        bmp.extend_from_slice(&pixels_per_meter.to_le_bytes());
        bmp.extend_from_slice(&colors_used.to_le_bytes());
        bmp.extend_from_slice(&colors_important.to_le_bytes());

        for row in (0..*sz).rev() {
            for col in 0..*sz {
                let i = ((row * sz) + col) * 4;
                bmp.push(raw[i as usize + 2]);
                bmp.push(raw[i as usize + 1]);
                bmp.push(raw[i as usize]);
                bmp.push(raw[i as usize + 3]);
            }
        }

        let and_mask_row_bytes = ((*sz + 31) / 32 * 4) as usize;
        let and_mask_size = and_mask_row_bytes * *sz as usize;
        bmp.extend_from_slice(&vec![0u8; and_mask_size]);

        image_data_blocks.push(bmp);

        let entry_size_total = 40u32 + image_size + and_mask_size as u32;

        f.write_all(&[w as u8, h as u8]).unwrap();
        f.write_all(&[0, 0]).unwrap();
        f.write_all(&color_planes.to_le_bytes()).unwrap();
        f.write_all(&bytes_per_pixel.to_le_bytes()).unwrap();
        f.write_all(&entry_size_total.to_le_bytes()).unwrap();
        f.write_all(&current_offset.to_le_bytes()).unwrap();

        current_offset += entry_size_total;
    }

    for block in &image_data_blocks {
        f.write_all(block).unwrap();
    }
}

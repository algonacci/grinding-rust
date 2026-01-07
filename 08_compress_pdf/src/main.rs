use lopdf::{Document, Object, Stream};
use image::{DynamicImage, ImageBuffer, imageops::FilterType, GenericImageView};
use mozjpeg::{Compress, ColorSpace};
use std::collections::HashSet;
use std::io::Read; 
use flate2::read::ZlibDecoder; 

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "input.pdf";
    let output = "output_compressed.pdf";
    
    let max_width = 1200;
    let jpeg_quality = 60.0;

    println!("📄 Loading PDF: {}", input);
    let mut doc = Document::load(input)?;

    let mut image_ids = HashSet::new();
    for (id, obj) in doc.objects.iter() {
        if is_image_xobject(obj) {
            image_ids.insert(*id);
        }
    }
    println!("🔍 Found {} images inside PDF", image_ids.len());

    let mut success_count = 0;
    let mut fail_count = 0;
    let keys: Vec<_> = image_ids.into_iter().collect();

    for object_id in keys {
        let filter_name = doc.get_object(object_id)
            .and_then(|o| o.as_stream())
            .map(|s| {
                s.dict.get(b"Filter").ok()
                 .and_then(|o| o.as_name_str().ok())
                 .unwrap_or("Unknown")
                 .to_string()
            }).unwrap_or("Error".to_string());

        let (raw_data, width, height, colorspace, bpc) = {
            let obj = doc.get_object(object_id)?;
            let stream = match obj.as_stream() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let width = stream.dict.get(b"Width").ok().and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let height = stream.dict.get(b"Height").ok().and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let bpc = stream.dict.get(b"BitsPerComponent").ok().and_then(|v| v.as_i64().ok()).unwrap_or(8) as u32;

            let cs = stream.dict.get(b"ColorSpace").ok()
                .and_then(|o| match o {
                    Object::Name(n) => std::str::from_utf8(n).ok(),
                    Object::Array(arr) => arr.get(0).and_then(|x| x.as_name_str().ok()),
                    _ => None
                }).unwrap_or("DeviceRGB").to_string();

            let data_res: Result<Vec<u8>, String> = if filter_name.contains("DCTDecode") {
                Ok(stream.content.clone())
            } else {
                match stream.decompressed_content() {
                    Ok(d) => Ok(d),
                    Err(_) => {
                        if filter_name.contains("FlateDecode") {
                            let mut decoder = ZlibDecoder::new(&stream.content[..]);
                            let mut buffer = Vec::new();
                            match decoder.read_to_end(&mut buffer) {
                                Ok(_) => Ok(buffer),
                                Err(e) => Err(format!("Manual Zlib Failed: {}", e))
                            }
                        } else {
                            Err("Unsupported Filter / Decode Failed".to_string())
                        }
                    }
                }
            };

            let data = match data_res {
                Ok(d) => d,
                Err(e) => {
                    if !e.contains("Unsupported") {
                         println!("   ❌ Failed extraction Img {}: {}", object_id.0, e);
                    }
                    fail_count += 1;
                    continue; 
                }
            };
            
            (data, width, height, cs, bpc)
        };

        if width == 0 || height == 0 {
           continue;
        }

        println!("➡️ Processing Img {} ({})", object_id.0, filter_name);

        let img_result = decode_pdf_image(&raw_data, width, height, &colorspace, bpc);

        match img_result {
            Ok(dynamic_img) => {
                match compress_image_logic(dynamic_img, max_width, jpeg_quality) {
                    Ok((compressed_data, new_w, new_h)) => {
                        let is_worth_it = compressed_data.len() < raw_data.len();
                        
                        // FIX: Simpan size dulu sebelum variable 'compressed_data' dipindahkan (moved)
                        let new_size = compressed_data.len(); 

                        if is_worth_it || filter_name.contains("FlateDecode") {
                            if let Ok(obj) = doc.get_object_mut(object_id) {
                                if let Ok(stream) = obj.as_stream_mut() {
                                    
                                    // Di sini ownership compressed_data pindah ke fungsi replace
                                    replace_stream_with_jpeg(stream, compressed_data, new_w, new_h);
                                    
                                    success_count += 1;
                                    
                                    // Pake variable 'new_size' yg kita simpan tadi
                                    println!("   ✨ Optimized: {}kb -> {}kb", raw_data.len()/1024, new_size/1024);
                                }
                            }
                        } else {
                            println!("   SKIP: Compressed is larger.");
                        }
                    },
                    Err(e) => {
                        println!("   ❌ Compression Error: {}", e);
                        fail_count += 1;
                    }
                }
            },
            Err(e) => {
                println!("   ❌ Decode Pixel Error: {} (CS: {})", e, colorspace);
                fail_count += 1;
            }
        }
    }

    doc.prune_objects();
    doc.save(output)?;
    
    println!("------------------------------------------------");
    println!("✅ Final: Optimized: {}, Failed: {}", success_count, fail_count);

    Ok(())
}

fn compress_image_logic(img: DynamicImage, max_width: u32, quality: f32) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let target_w = if img.width() > max_width { max_width } else { img.width() };
    let resized_img = img.resize(target_w, u32::MAX, FilterType::Lanczos3);
    let (w, h) = resized_img.dimensions();
    let rgb_img = resized_img.to_rgb8();

    let mut comp = Compress::new(ColorSpace::JCS_RGB);
    comp.set_size(w as usize, h as usize);
    comp.set_quality(quality);
    let mut comp_buf = Vec::new();
    let mut compressor = comp.start_compress(&mut comp_buf)?;
    compressor.write_scanlines(rgb_img.as_raw())?;
    compressor.finish()?;
    Ok((comp_buf, w, h))
}

// FIX: Tambah underscore _bpc biar ga warning unused variable
fn decode_pdf_image(data: &[u8], width: u32, height: u32, cs: &str, _bpc: u32) -> Result<DynamicImage, String> {
    if let Ok(img) = image::load_from_memory(data) {
        return Ok(img);
    }
    
    let size = (width * height) as usize;

    if cs.contains("DeviceRGB") || cs.contains("RGB") {
        if data.len() < size * 3 { return Err(format!("Data length mismatch for RGB. Need {}, got {}", size*3, data.len())); }
        let buf = ImageBuffer::from_raw(width, height, data[..size*3].to_vec()).ok_or("Failed to create RGB buffer")?;
        return Ok(DynamicImage::ImageRgb8(buf));
    } 
    else if cs.contains("DeviceGray") || cs.contains("Gray") {
        if data.len() < size { return Err(format!("Data length mismatch for Gray. Need {}, got {}", size, data.len())); }
        let buf = ImageBuffer::from_raw(width, height, data[..size].to_vec()).ok_or("Failed to create Gray buffer")?;
        return Ok(DynamicImage::ImageLuma8(buf));
    }
    else if cs.contains("DeviceCMYK") || cs.contains("CMYK") {
        if data.len() < size * 4 { return Err("Not enough data for CMYK".into()); }
        let mut rgb_data = Vec::with_capacity(size * 3);
        for chunk in data.chunks(4) {
            if chunk.len() < 4 { break; }
            let c = chunk[0] as f32 / 255.0;
            let m = chunk[1] as f32 / 255.0;
            let y = chunk[2] as f32 / 255.0;
            let k = chunk[3] as f32 / 255.0;
            let r = (255.0 * (1.0 - c) * (1.0 - k)) as u8;
            let g = (255.0 * (1.0 - m) * (1.0 - k)) as u8;
            let b = (255.0 * (1.0 - y) * (1.0 - k)) as u8;
            rgb_data.push(r); rgb_data.push(g); rgb_data.push(b);
        }
        let buf = ImageBuffer::from_raw(width, height, rgb_data).ok_or("Failed to create RGB buffer from CMYK")?;
        return Ok(DynamicImage::ImageRgb8(buf));
    }

    Err(format!("Unsupported Colorspace: {}", cs))
}

fn replace_stream_with_jpeg(stream: &mut Stream, data: Vec<u8>, w: u32, h: u32) {
    stream.set_content(data);
    stream.dict.set("Type", "XObject");
    stream.dict.set("Subtype", "Image");
    stream.dict.set("Filter", "DCTDecode");
    stream.dict.set("ColorSpace", "DeviceRGB");
    stream.dict.set("BitsPerComponent", 8);
    stream.dict.set("Width", w as i64);
    stream.dict.set("Height", h as i64);
    stream.dict.remove(b"DecodeParms");
    stream.dict.remove(b"FilterParms");
    stream.dict.remove(b"Length");
    stream.dict.remove(b"Predictor");
    stream.dict.remove(b"Columns");
}

fn is_image_xobject(obj: &Object) -> bool {
    if let Object::Stream(stream) = obj {
        return stream.dict.get(b"Subtype").ok() 
            .and_then(|o| o.as_name_str().ok())
            .map(|s| s == "Image")
            .unwrap_or(false);
    }
    false
}
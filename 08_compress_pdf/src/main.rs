use lopdf::{Document, Object, Dictionary, Stream};
use image::{DynamicImage, imageops::FilterType};
use mozjpeg::{Compress, ColorSpace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "input.pdf";
    let output = "output_compressed.pdf";

    println!("Loading PDF from {}...", input);
    let mut doc = Document::load(input)?;

    let image_ids = collect_image_xobjects(&doc);
    println!("Found {} image XObjects", image_ids.len());

    let mut optimized = 0;

    for id in image_ids {
        if let Ok(obj) = doc.get_object_mut(id) {
            // Must handle Object::Stream specifically
            if let Object::Stream(stream) = obj {
                 match process_image_stream(stream) {
                    Ok((jpeg, w, h)) => {
                        replace_with_jpeg(stream, jpeg, w, h);
                        optimized += 1;
                    }
                    Err(e) => {
                        // println!("Failed to process image {}: {}", id.0, e);
                    }
                }
            }
        }
    }

    // Remove unused objects to save space
    doc.prune_objects();
    
    // Fix Cross-Reference Table - handled by save automatically
    // doc.reference_table_ids = doc.objects.keys().cloned().collect();

    println!("Optimized {} images", optimized);
    println!("Saving to {}...", output);
    doc.save(output)?;

    println!("Done 🔥");
    Ok(())
}

/// Collect IDs of all XObjects that are Subtype=Image
fn collect_image_xobjects(doc: &Document) -> Vec<lopdf::ObjectId> {
    let mut ids = Vec::new();
    for (id, obj) in doc.objects.iter() {
        if let Object::Stream(stream) = obj {
            // Check dictionary for /Subtype /Image
            let is_image = stream.dict.get(b"Subtype")
                .and_then(|t| t.as_name_str())
                .map(|s| s == "Image")
                .unwrap_or(false);
            
            // Check /Type /XObject (optional but good practice)
            let is_xobject = stream.dict.get(b"Type")
                .and_then(|t| t.as_name_str())
                .map(|s| s == "XObject")
                .unwrap_or(true); // Some PDFs omit Type in stream dict

            if is_image && is_xobject {
                ids.push(*id);
            }
        }
    }
    ids
}

/// Decode -> Resize -> MozJPEG Encode
fn process_image_stream(
    stream: &Stream,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let data = stream.decompressed_content()
        .map_err(|_| "Failed to decompress")?;

    // Try to determine format to load into Image crate
    let mut img = image::load_from_memory(&data).ok();
    
    // Fallback: Try decoding raw if we have dimensions
    if img.is_none() {
        // Safe parsing of dictionary properties
        let width = stream.dict.get(b"Width")
            .and_then(|o| o.as_i64())
            .unwrap_or(0) as u32;
            
        let height = stream.dict.get(b"Height")
            .and_then(|o| o.as_i64())
            .unwrap_or(0) as u32;
            
        let cs = stream.dict.get(b"ColorSpace")
            .and_then(|o| o.as_name_str())
            .unwrap_or("");
        
        if width > 0 && height > 0 {
            if cs == "DeviceRGB" && data.len() as u32 == width * height * 3 {
                 img = image::RgbImage::from_raw(width, height, data)
                    .map(DynamicImage::ImageRgb8);
            } else if cs == "DeviceGray" && data.len() as u32 == width * height {
                 img = image::GrayImage::from_raw(width, height, data)
                    .map(DynamicImage::ImageLuma8);
            }
        }
    }

    let mut dynamic_image = img.ok_or("Could not decode image")?;

    // Resize if too large (Aggressive: 1200px max)
    if dynamic_image.width() > 1200 {
        dynamic_image = dynamic_image.resize(1200, u32::MAX, FilterType::Lanczos3);
    }
    
    let rgb = dynamic_image.to_rgb8();
    let (w, h) = rgb.dimensions();

    // MozJPEG Encoding (Aggressive)
    let mut comp = Compress::new(ColorSpace::JCS_RGB);
    comp.set_size(w as usize, h as usize);
    comp.set_quality(50.0); // Low quality for max compression
    // comp.set_subsampling(Subsampling::Subsampling420); // Not available in 0.10 SAFE API? 
    // Actually set_subsampling might be unsafe or require different struct. 
    // Let's stick to safe defaults or check crate docs. 
    // set_mem_dest is deprecated, start_compress(&mut vec) is the way.

    let mut comp_buf = Vec::new();
    let mut compressor = comp.start_compress(&mut comp_buf)?;
    compressor.write_scanlines(rgb.as_raw())?;
    compressor.finish()?;

    Ok((comp_buf, w, h))
}

/// Replace PDF stream with new JPEG data
fn replace_with_jpeg(
    stream: &mut Stream,
    jpeg: Vec<u8>,
    width: u32,
    height: u32,
) {
    stream.set_content(jpeg);

    // Reset dictionary to be a pure JPEG XObject
    stream.dict.set("Type", "XObject");
    stream.dict.set("Subtype", "Image");
    stream.dict.set("Filter", "DCTDecode");
    stream.dict.set("ColorSpace", "DeviceRGB");
    stream.dict.set("BitsPerComponent", 8);
    stream.dict.set("Width", width as i64);
    stream.dict.set("Height", height as i64);

    // Remove legacy filters
    stream.dict.remove(b"DecodeParms");
    stream.dict.remove(b"FilterParms");
    stream.dict.remove(b"Length"); // lopdf updates this automatically on save
    stream.dict.remove(b"Mask");   // Opaque JPEG doesn't support basic masks
    stream.dict.remove(b"SMask");
}

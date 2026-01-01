use image::{ImageReader, GenericImageView};
use mozjpeg::{Compress, ColorSpace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "input.png";     // ganti nama file
    let output = "output.jpg";   // hasil kecil tapi cakep

    optimize_thumbnail(input, output, 82.0)?;

    println!("Done! Thumbnail optimized 🔥");
    Ok(())
}

fn optimize_thumbnail(
    input: &str,
    output: &str,
    quality: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // load image
    let img = ImageReader::open(input)?.decode()?;

    let (w, h) = img.dimensions();
    let img_rgb = img.to_rgb8();

    // jpeg encode
    let mut comp = Compress::new(ColorSpace::JCS_RGB);
    comp.set_size(w as usize, h as usize);
    comp.set_quality(quality);
    
    let mut comp_buf = Vec::new();
    let mut compressor = comp.start_compress(&mut comp_buf)?;
    
    compressor.write_scanlines(img_rgb.as_raw())?;
    
    compressor.finish()?;

    std::fs::write(output, comp_buf)?;
    Ok(())
}

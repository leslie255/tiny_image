use tiny_image::*;

macro_rules! time {
    ($label:literal, $expr:expr) => {{
        use std::time::Instant;
        let before = Instant::now();
        $expr;
        let after = Instant::now();
        let duration = after.duration_since(before);
        let seconds = duration.as_secs_f64();
        println!("{}: {seconds} seconds", $label);
    }};
}

fn main() {
    use std::fs;

    fn checkerboard_pattern(x: u32, y: u32) -> [u8; 3] {
        match (x / 32 + y / 32).is_multiple_of(2) {
            true => [128, 192, 255],
            false => [255, 192, 128],
        }
    }

    let image = ImageBuffer::<Rgb8U>::from_fn(256, 256, checkerboard_pattern);

    let mut out_buffer = Vec::new();

    time!("QOI encoding", image.encode_qoi(&mut out_buffer));
    fs::write("foo/image.qoi", &out_buffer).unwrap();

    out_buffer.clear();

    time!("PNG encoding", image.encode_png(&mut out_buffer));
    fs::write("foo/image.png", &out_buffer).unwrap();

    println!("Image:");
    image.print_with_kitty_graphics().unwrap();
    println!();

    match fs::read("foo/image.png") {
        Ok(data) => {
            let decoded_image = AnyImageBuffer::decode_from_png(&data).unwrap();
            dbg!(decoded_image.format());
            let image_rgba8 = decoded_image
                .into_format_lossy::<Rgba32F>()
                .into_format_lossy::<Luma8U>()
                .into_format_lossy::<Rgba8U>();
            println!("Image grayscaled:");
            image_rgba8.print_with_kitty_graphics().unwrap();
            println!();

            out_buffer.clear();
            image_rgba8.encode_png(&mut out_buffer);
            fs::write("foo/image_reencoded.png", &out_buffer).unwrap();
        }
        Err(error) => {
            println!("Cannot read foo/image.png: {error}");
        }
    }
}


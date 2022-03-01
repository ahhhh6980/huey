mod color;
use image::Delay;
use image::{codecs::gif::GifEncoder, Rgba};
use color::Color;
use std::fs::File;
use std::{
    error::Error,
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
    time::Instant,
};

#[allow(dead_code)]
enum FindType {
    File,
    Dir,
}

fn list_dir<P: AsRef<Path>>(dir: P, find_dirs: FindType) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::<PathBuf>::new();
    for item in fs::read_dir(dir)? {
        let item = item?;
        match &find_dirs {
            FindType::File => {
                if item.file_type()?.is_file() {
                    files.push(item.path());
                }
            }
            FindType::Dir => {
                if item.file_type()?.is_dir() {
                    files.push(item.path());
                }
            }
        }
    }
    Ok(files)
}

fn prompt_number(bounds: Range<u32>, message: &str) -> io::Result<u32> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    // Tell the user to enter a value within the bounds
    if message != "" {
        println!(
            "{} in the range [{}:{}]",
            message,
            bounds.start,
            bounds.end - 1
        );
    }
    // Keep prompting until the user passes a value within the bounds
    Ok(loop {
        stdin.read_line(&mut buffer)?;
        if let Ok(value) = buffer.trim().parse() {
            if bounds.contains(&value) {
                break value;
            }
        }
        buffer.clear();
    })
}

fn input_prompt<P: AsRef<Path>>(
    dir: P,
    find_dirs: FindType,
    message: &str,
) -> std::io::Result<PathBuf> {
    // Get files/dirs in dir
    let files = list_dir(&dir, find_dirs)?;
    // Inform the user that they will need to enter a value
    if message != "" {
        println!("{}", message);
    }
    // Enumerate the names of the files/dirs
    for (i, e) in files.iter().enumerate() {
        println!("{}: {}", i, e.display());
    }
    // This is the range of values they can pick
    let bound: Range<u32> = Range {
        start: 0,
        end: files.len() as u32,
    };
    // Return the path they picked
    Ok((&files[prompt_number(bound, "")? as usize]).clone())
}

fn main() -> Result<(), Box<dyn Error>> {
    let fname = input_prompt("input", FindType::File, "Choose the input image")?;
    let colors = prompt_number(
        Range {
            start: 10,
            end: 361,
        },
        "Choose the number of colors",
    )?;
    let msg = format!("Choose frame rate");
    let frame_time = prompt_number(Range { start: 0, end: 61 }, &msg)?;
    let input_name = String::from(
        fname
            .file_name()
            .unwrap()
            .to_string_lossy()
            .split(".")
            .collect::<Vec<&str>>()[0],
    );
    let ext = String::from(
        fname
            .file_name()
            .unwrap()
            .to_string_lossy()
            .split(".")
            .collect::<Vec<&str>>()[1],
    );
    let frame_path = format!("{p}/{p}{}", "_frames", p = &input_name);
    let gifname = format!("{p}/{p}.gif", p = &input_name);

    if Path::new(&input_name).exists() == false {
        fs::create_dir(&input_name)?;
    }
    if Path::new(&frame_path).exists() {
        fs::remove_dir_all(&frame_path)?;
    }
    fs::create_dir(&frame_path)?;

    // , n=&input_name
    let message = format!("ffmpeg -i {f}_frames/%d_{f}.{e} -vf palettegen palette.png && ffmpeg -v warning -i {f}_frames/%d_{f}.{e} -i palette.png  -lavfi \"paletteuse,setpts=N/({d}*TB)\" -y {f}.gif && rm palette.png", f=&input_name, d=frame_time, e=&ext);
    let message_path = format!("{}/ffmpeg_command.txt", &input_name);
    fs::write(&message_path, &message)?;

    let now = Instant::now();
    for h in 0..colors {
        let mut image = image::open(&fname)?.to_rgba8();
        for (x, y, pixel) in image::open(&fname)?.to_rgba32f().enumerate_pixels() {
            let mut color = Color {
                r: pixel.0[0],
                g: pixel.0[1],
                b: pixel.0[2],
                a: pixel.0[3],
                mode: color::ColorType::RGBA,
            }
            .to_HSVA();
            if color.a != 1.0 {
                color.a = 0.0;
            }
            color.r = (color.r + (h as f32 * (360.0 / colors as f32))) % 360.0;
            let color = Rgba::<u8>::from(color.to_RGBA().to_arr8());
            image.put_pixel(x, y, color);
        }
        let oname = format!("{}/{}_{}.{}", &frame_path, h, &input_name, &ext);
        println!(
            "| {:.2}% | Processing {} |",
            (h as f32 / (2.0 * colors as f32)) * 100.0,
            &oname
        );
        image.save(oname)?;
    }

    if Path::new(&gifname).exists() {
        fs::remove_file(&gifname)?;
    }

    let image = File::create(&gifname)?;
    let mut gif = GifEncoder::new(&image);
    gif.set_repeat(image::codecs::gif::Repeat::Infinite)?;

    for h in 0..colors {
        let oname = format!("{}/{}_{}.{}", &frame_path, h, &input_name, &ext);
        println!(
            "| {:.2}% | Encoding {} |",
            ((h as f32 / (2.0 * colors as f32)) * 100.0) + 50.0,
            &oname
        );
        let pixels = image::open(&oname)?.to_rgba8();
        let frame =
            image::Frame::from_parts(pixels, 0, 0, Delay::from_numer_denom_ms(frame_time, colors));

        frame.delay();
        gif.encode_frame(frame)?;
    }

    println!("Finished in: {:.2}s!", now.elapsed().as_secs_f32());
    Ok(())
}

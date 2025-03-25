use std::{path::{Path, PathBuf}, str::FromStr, fs};

use clap::Parser;
use image_dds::Mipmaps;
use ultimate_tex_lib::{ImageFile, NutexbFile};

#[derive(Parser, Debug)]
#[command(author, version, about = "Smash Ultimate texture converter", long_about = None)]
struct Args {
    #[arg(help = "The input image file to convert")]
    input: String,

    #[arg(help = "The output converted image file")]
    output: String,

    // TODO: make this a value enum to show possible image formats?
    #[arg(
        short = 'f',
        long = "format",
        help = "The output image format for files supporting compression"
    )]
    format: Option<String>,

    #[arg(
        long = "no-mipmaps",
        help = "Disable mipmap generation and only include the base mip level"
    )]
    no_mipmaps: bool,
}

fn main() {
    let args = Args::parse();
    let input = Path::new(&args.input);
    
    // Process output path to handle * replacement
    let output_path_str = if args.output.contains('*') {
        // Only attempt to replace * if input is a nutexb file
        if input.extension().map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "nutexb") {
            // Read the nutexb file to get the internal name
            let nutexb = NutexbFile::read_from_file(input).unwrap();
            let internal_name = nutexb.footer.string.to_string();
            args.output.replace('*', &internal_name)
        } else {
            // If not a nutexb file, keep the * as is
            args.output.clone()
        }
    } else {
        args.output.clone()
    };
    
    let output = Path::new(&output_path_str);
    
    let input_image = ImageFile::from_file(input).unwrap();

    let format = args
        .format
        .map(|s| image_dds::ImageFormat::from_str(&s).unwrap())
        .unwrap_or(image_dds::ImageFormat::BC7RgbaUnorm);

    let quality = image_dds::Quality::Fast;

    let mipmaps = if args.no_mipmaps {
        Mipmaps::Disabled
    } else {
        Mipmaps::GeneratedAutomatic
    };

    let output_extension = output
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_lowercase();
    
    match output_extension.as_str() {
        "nutexb" => {
            input_image
                .save_nutexb(output, format, quality, mipmaps)
                .unwrap();
            // Print NutexbFooter info for output file
            if let Ok(nutexb) = NutexbFile::read_from_file(output) {
                println!("\nNutexbFooter Information:");
                println!("Name: {}", nutexb.footer.string);
                println!("Dimensions: {}x{}x{}", nutexb.footer.width, nutexb.footer.height, nutexb.footer.depth);
                println!("NutexbFormat: {:?}", nutexb.footer.image_format);
                println!("ImageFormat: {:?}", ultimate_tex_lib::nutexb_image_format(&nutexb));
                println!("Mipmap Count: {}", nutexb.footer.mipmap_count);
                println!("Layer Count: {}", nutexb.footer.layer_count);
                println!("Data Size: {} bytes", nutexb.footer.data_size);
            }
        }
        "bntx" => input_image
            .save_bntx(output, format, quality, mipmaps)
            .unwrap(),
        "dds" => input_image
            .save_dds(output, format, quality, mipmaps)
            .unwrap(),
        // For image formats, use our function to ensure unique filenames
        _ => {
            let unique_path = ensure_unique_filename(output);
            input_image.save_image(&unique_path).unwrap()
        },
    };

    // Print NutexbFooter info if input was a nutexb file
    if input.extension().map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "nutexb") {
        if let Ok(nutexb) = NutexbFile::read_from_file(input) {
            println!("\nInput NutexbFooter Information:");
            println!("Name: {}", nutexb.footer.string);
            println!("Dimensions: {}x{}x{}", nutexb.footer.width, nutexb.footer.height, nutexb.footer.depth);
            println!("Format: {:?}", ultimate_tex_lib::nutexb_image_format(&nutexb));
            println!("Mipmap Count: {}", nutexb.footer.mipmap_count);
            println!("Layer Count: {}", nutexb.footer.layer_count);
            println!("Data Size: {} bytes", nutexb.footer.data_size);
        }
    }
}

/// Ensures a unique filename by checking if the file already exists
/// and adding a numbered suffix if necessary.
fn ensure_unique_filename(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    
    // Get the file stem and extension
    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
    let ext = path.extension().unwrap().to_string_lossy().to_string();
    let parent = path.parent().unwrap_or(Path::new(""));
    
    // Try adding _1, _2, etc. until we find a unique name
    let mut counter = 1;
    loop {
        let new_name = format!("{}_{}.{}", stem, counter, ext);
        let new_path = parent.join(new_name);
        
        if !new_path.exists() {
            return new_path;
        }
        
        counter += 1;
    }
}

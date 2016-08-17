// This example implements an OCR (optical character recognition):
// https://en.wikipedia.org/wiki/Optical_character_recognition
// using an evolutionary algorithm.
//
// Note that evolutionary algorithms do no guarantee to always find the optimal solution.
// But they can get very close.


extern crate rand;
extern crate image;
extern crate imageproc;
extern crate rusttype;
#[macro_use] extern crate lazy_static;
extern crate simplelog;

// internal crates
extern crate darwin_rs;

use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use image::{GenericImage, ImageBuffer, Luma};
use image::imageops::replace;
use imageproc::stats::root_mean_squared_error;
use simplelog::{SimpleLogger, LogLevelFilter};
use rusttype::{FontCollection};

// internal modules
use darwin_rs::{Individual, SimulationBuilder, PopulationBuilder, SimError};

#[derive(Debug, Clone)]
struct TextBox {
    x: u32,
    y: u32,
    text: String,
}

#[derive(Debug, Clone)]
struct OCRItem {
    content: Vec<TextBox>,
    // put inside lazy_static: Box<ImageBuffer<Luma<u8>, Vec<u8>>>,
}

impl Individual for OCRItem {
    fn new() -> OCRItem {
        OCRItem { content: Vec::new() }
    }

    fn mutate(&mut self) {}

    fn calculate_fitness(&self) -> f64 {
        0.0
    }
}

fn draw_text_line(canvas: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
                  font: &rusttype::Font,
                  pos_x: i32,
                  pos_y: i32,
                  text: &str) {

    let height: f32 = 18.0;
    let pixel_height = height.ceil() as u32;

    let scale = rusttype::Scale { x: height * 1.0, y: height };
    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);
    let glyphs: Vec<rusttype::PositionedGlyph> = font.layout(text, scale, offset).collect();

/*
    let width = glyphs.iter().rev()
            .filter_map(|g| g.pixel_bounding_box()
            .map(|b| b.min.x as f32 + g.unpositioned().h_metrics().advance_width))
            .next().unwrap_or(0.0).ceil() as u32;
*/

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = ((x as i32) + bb.min.x + pos_x) as u32;
                let y = ((y as i32) + bb.min.y + pos_y) as u32;
                if x >= 0 && y >= 0 && x <= canvas.width() && y <= canvas.height() {
                    canvas.put_pixel(x, y, Luma::<u8>{ data: [(v * 255.0) as u8] } );
                }
            })
        }
    }

}

fn main() {
    println!("Darwin test: optical character recognition");

    let _ = SimpleLogger::init(LogLevelFilter::Info);

    let mut original_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(640, 70);
    let mut contructed_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(640, 70);

    let mut file = File::open("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf").unwrap();
    let mut font_data: Vec<u8> = Vec::new();
    let bytes_read = file.read_to_end(&mut font_data).unwrap();
    println!("bytes read: {}", bytes_read);

    let collection = rusttype::FontCollection::from_bytes(font_data);
    let font = collection.into_font().unwrap();

    draw_text_line(&mut original_img, &font, 10, 10, "This is a test text!");
    draw_text_line(&mut original_img, &font, 10, 40,
        "Just to see how good OCR works...");

    let img_file = Path::new("rendered_text.png");
    let _ = original_img.save(&img_file);


    let population1 = PopulationBuilder::<OCRItem>::new()
        .set_id(1)
        .individuals(100)
        .increasing_exp_mutation_rate(1.03)
        .reset_limit_increment(100)
        .reset_limit_start(100)
        .reset_limit_end(1000)
        .finalize().unwrap();

    let population2 = PopulationBuilder::<OCRItem>::new()
        .set_id(2)
        .individuals(100)
        .increasing_exp_mutation_rate(1.04)
        .reset_limit_increment(200)
        .reset_limit_start(100)
        .reset_limit_end(2000)
        .finalize().unwrap();


    let ocr_builder = SimulationBuilder::<OCRItem>::new()
        .fitness(10.0)
        .threads(2)
        .add_population(population1)
        .add_population(population2)
        .finalize();

    match ocr_builder {
        Err(SimError::EndIterationTooLow) => println!("more than 10 iteratons needed"),
        Ok(mut tsp_simulation) => {
            // tsp_simulation.run();
            //
            // println!("total run time: {} ms", tsp_simulation.total_time_in_ms);
            // println!("improvement factor: {}", tsp_simulation.improvement_factor);
            // println!("number of iterations: {}", tsp_simulation.iteration_counter);
            //
            // tsp_simulation.print_fitness();
            //
        }
    }
}

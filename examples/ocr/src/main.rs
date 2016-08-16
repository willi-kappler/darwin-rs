// This example implements an OCR (optical character recognition):
// https://en.wikipedia.org/wiki/Optical_character_recognition
// using an evolutionary algorithm.
//
// Note that evolutionary algorithms do no guarantee to always find the optimal solution.
// But they can get very close.


extern crate rand;
extern crate image;
extern crate imageproc;
extern crate freetype;
#[macro_use] extern crate lazy_static;

// internal crates
extern crate darwin_rs;

use rand::Rng;
use std::fs::File;
use std::path::Path;
use image::{GenericImage, ImageBuffer, Luma};
use image::imageops::replace;
use imageproc::stats::root_mean_squared_error;

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
                  face: &freetype::Face,
                  x: u32,
                  y: u32,
                  text: &str) {
    let mut pos_x = x;
    let pos_y = y;

    for char in text.chars() {
        // println!("processing char: {}, pos_x: {}", char, pos_x);
        face.load_char(char as usize, freetype::face::RENDER).unwrap();
        let glyph = face.glyph();
        let bm = glyph.bitmap();

        if !char.is_whitespace() {
            let bm_slice = bm.buffer().to_vec();

            let rendered_char: ImageBuffer<Luma<u8>, Vec<u8>> =
                ImageBuffer::from_vec(bm.pitch() as u32, bm.rows() as u32, bm_slice).unwrap();
            replace(canvas,
                    &rendered_char,
                    pos_x + (glyph.bitmap_left() as u32),
                    pos_y - (glyph.bitmap_top() as u32));
        }

        let step_x = ((glyph.get_glyph().unwrap().advance_x()) >> 16) as u32;

        pos_x = pos_x + step_x;
    }
}

fn main() {
    println!("Darwin test: optical character recognition");

    let mut original_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(640, 70);
    let mut contructed_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(640, 70);

    let ft_library = freetype::Library::init().unwrap();
    let face = ft_library.new_face("/usr/share/fonts/truetype/freefont/FreeMono.ttf", 0).unwrap();
    face.set_char_size(40 * 64, 0, 50, 0).unwrap();

    draw_text_line(&mut original_img, &face, 10, 30, "This is a test text!");
    draw_text_line(&mut original_img,
                   &face,
                   10,
                   60,
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

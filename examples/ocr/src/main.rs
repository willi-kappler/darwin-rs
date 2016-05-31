extern crate rand;
extern crate image;
extern crate imageproc;
extern crate freetype;

// internal crates
extern crate darwin_rs;

use rand::Rng;
use std::fs::File;
use std::path::Path;
use image::{GenericImage, ImageBuffer, Rgb};
use imageproc::stats::root_mean_squared_error;
use freetype::RenderMode;

#[derive(Debug, Clone)]
struct OCRItem {
    text: String
}

impl Individual for OCRItem {
    fn new() -> OCRItem {
        OCRItem{ text: "abc test".to_string() }
    }

    fn mutate(&mut self) {
    }

    fn calculate_fittness(&self) -> f64 {
        0.0
    }
}

// internal modules
use darwin_rs::{Individual, SimulationBuilder, BuilderResult};

fn main() {
    println!("Darwin test: optical character recognition");

    let tsp_builder = SimulationBuilder::<OCRItem>::new()
        .factor(0.34)
        .threads(2)
        .individuals(100)
        .sorting_fittest()
        .increasing_exp_mutation_rate(1.03)
        .finalize();

        match tsp_builder {
            BuilderResult::TooLowEndIterration => { println!("more than 10 iteratons needed") },
            BuilderResult::TooLowIndividuals => { println!("more than 2 individuals needed") },
            BuilderResult::InvalidFittestCount => { println!("number of random fittest count > number of individuals") },
            BuilderResult::Ok(mut tsp_simulation) => {
                /*
                tsp_simulation.run();

                println!("total run time: {} ms", tsp_simulation.total_time_in_ms);
                println!("improvement factor: {}", tsp_simulation.improvement_factor);
                println!("number of iterations: {}", tsp_simulation.iteration_counter);

                tsp_simulation.print_fittness();
                */
            }
        }

//        let img1 = image::open(&Path::new("ocr1.png")).unwrap();
//        let img2 = image::open(&Path::new("ocr2.png")).unwrap();
        let mut img3: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(640, 480);

//        println!("dimensions1 {:?}", img1.dimensions());
//        println!("dimensions2 {:?}", img2.dimensions());
        println!("dimensions3 {:?}", img3.dimensions());

//        println!("color1: {:?}", img1.color());
//        println!("color2: {:?}", img2.color());
        // println!("color3: {:?}", img3.color());

//        let error = root_mean_squared_error(&img1, &img2);

//        println!("error1: {}", error);

        let ft_library = freetype::Library::init().unwrap();
        let face = ft_library.new_face("/usr/share/fonts/truetype/freefont/FreeMono.ttf", 0).unwrap();
        face.set_char_size(40 * 64, 0, 50, 0).unwrap();
        face.load_char(65, freetype::face::RENDER | freetype::face::TARGET_LCD).unwrap();
        let glyph = face.glyph();
        // glyph.render_glyph(RenderMode::Lcd);
        let x = glyph.bitmap_left() as usize;
        let y = glyph.bitmap_top() as usize;
        let bm = glyph.bitmap();
        let bm_slice = bm.buffer().to_vec();

        println!("x: {}, y: {}, width: {}, rows: {}, len: {}, pitch: {}", x, y, bm.width(), bm.rows(), bm_slice.len(), bm.pitch());
        println!("pixel mode: {:?}", bm.pixel_mode());

        let img4: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(17, 16, bm_slice ).unwrap();
        let fout = Path::new("char1.png");
        let _ = img4.save(&fout);
}

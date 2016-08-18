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
extern crate simplelog;

// internal crates
extern crate darwin_rs;

use std::sync::Arc;
use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use image::{ImageBuffer, Luma};
use imageproc::stats::root_mean_squared_error;
use simplelog::{SimpleLogger, LogLevelFilter};
use std::str;

// internal modules
use darwin_rs::{Individual, SimulationBuilder, PopulationBuilder, SimError};

fn make_population<'a>(count: u32, config: &OCRConfig<'a>) -> Vec<FindPos1<'a>> {
    let mut result = Vec::new();

    let shared = Arc::new(config.clone());

    for _ in 0..count {
        result.push( FindPos1 {
            x: 0,
            y: 0,
            config: shared.clone()
        });
    }

    result
}

#[derive(Clone)]
struct OCRConfig<'a> {
    font: rusttype::Font<'a>,
    original_img: ImageBuffer<Luma<u8>, Vec<u8>>
}

#[derive(Clone)]
struct FindPos1<'a> {
    x: u32,
    y: u32,
    config: Arc<OCRConfig<'a>>
}

impl<'a> Individual for FindPos1<'a> {
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();

        self.x = rng.gen_range(0, self.config.original_img.width());
        self.y = rng.gen_range(0, self.config.original_img.height());
    }

    fn calculate_fitness(&self) -> f64 {
        let mut constructed_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(640, 70);

        draw_text_line(&mut constructed_img, &self.config.font, self.x as i32, self.y as i32, "T");

        root_mean_squared_error(&self.config.original_img, &constructed_img)
    }

    fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
    }

}



fn draw_text_line(canvas: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
                  font: &rusttype::Font,
                  pos_x: i32,
                  pos_y: i32,
                  text: &str) {

    let height: f32 = 18.0;

    let scale = rusttype::Scale { x: height * 1.0, y: height };
    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);
    let glyphs: Vec<rusttype::PositionedGlyph> = font.layout(text, scale, offset).collect();

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = ((x as i32) + bb.min.x + pos_x) as u32;
                let y = ((y as i32) + bb.min.y + pos_y) as u32;
                if x < canvas.width() && y < canvas.height() {
                    canvas.put_pixel(x, y, Luma::<u8>{ data: [(v * 255.0) as u8] } );
                }
            })
        }
    }

}

fn main() {
    println!("Darwin test: optical character recognition");

    let _ = SimpleLogger::init(LogLevelFilter::Info);


    // TODO: use fontconfig-rs in the future: https://github.com/abonander/fontconfig-rs
    let mut file = File::open("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf").unwrap();
    let mut font_data: Vec<u8> = Vec::new();
    let bytes_read = file.read_to_end(&mut font_data).unwrap();
    // println!("bytes read: {}", bytes_read);

    let collection = rusttype::FontCollection::from_bytes(font_data);
    let font = collection.into_font().unwrap();

    let mut original_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(640, 70);
    draw_text_line(&mut original_img, &font, 10, 10, "This is a test text!");
    draw_text_line(&mut original_img, &font, 10, 40, "Just to see how good OCR works...");

    let img_file = Path::new("rendered_text.png");
    let _ = original_img.save(&img_file);

    let ocr_config = OCRConfig { font: font, original_img: original_img };

    let initial_population = make_population(50, &ocr_config);

    let population1 = PopulationBuilder::<FindPos1>::new()
        .set_id(1)
        .initial_population(&initial_population)
        .increasing_exp_mutation_rate(1.02)
        .reset_limit_increment(100)
        .reset_limit_start(100)
        .reset_limit_end(5000)
        .finalize().unwrap();

    let ocr_builder = SimulationBuilder::<FindPos1>::new()
        .fitness(34.14)
        .threads(1)
        .add_population(population1)
        .finalize();

    match ocr_builder {
        Err(SimError::EndIterationTooLow) => println!("more than 10 iteratons needed"),
        Ok(mut ocr_simulation) => {
            ocr_simulation.run();

            println!("total run time: {} ms", ocr_simulation.total_time_in_ms);
            println!("improvement factor: {}", ocr_simulation.simulation_result.improvement_factor);
            println!("number of iterations: {}", ocr_simulation.simulation_result.iteration_counter);

            let ref pos = ocr_simulation.simulation_result.fittest[0].individual;
            println!("x: {}, y: {}", pos.x, pos.y);

        }
    }
}

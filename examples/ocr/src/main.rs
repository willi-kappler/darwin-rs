// This example implements an OCR (optical character recognition):
// https://en.wikipedia.org/wiki/Optical_character_recognition
// using an evolutionary algorithm.
//
// Note that evolutionary algorithms do no guarantee to always find the optimal solution.
// But they can get very close.


extern crate rand;
#[macro_use] extern crate log;
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

fn make_population<'a>(count: u32, config: &OCRConfig<'a>) -> Vec<OCRItem<'a>> {
    let mut result = Vec::new();

    let shared = Arc::new(config.clone());

    for _ in 0..count {
        result.push( OCRItem {
            content: vec![
                // Start with letter 'A' in each line
                TextBox{ x: 10, y: 10, text: vec![65, 65, 65, 65, 65, 65, 65, 65, 65] },
                TextBox{ x: 10, y: 40, text: vec![65, 65, 65, 65, 65, 65, 65, 65, 65] }],
            config: shared.clone()
        });
    }

    result
}

#[derive(Clone)]
struct TextBox {
    x: u32,
    y: u32,
    text: Vec<u8>
}

#[derive(Clone)]
struct OCRConfig<'a> {
    font: rusttype::Font<'a>,
    original_img: ImageBuffer<Luma<u8>, Vec<u8>>
}

#[derive(Clone)]
struct OCRItem<'a> {
    content: Vec<TextBox>,
    config: Arc<OCRConfig<'a>>
}

impl<'a> Individual for OCRItem<'a> {
    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();

        let content_line = rng.gen_range(0, 2);

        let operation = rng.gen_range(0, 2);

        let index1 = rng.gen_range(0, self.content[content_line].text.len());

        match operation {
            0 => {
                // Change character
                let new_char = rng.gen_range(32, 127); // All printable ASCII characters
                self.content[content_line].text[index1] = new_char;
            },
            1 => {
                // Swap characters
                let index2 = rng.gen_range(0, self.content[content_line].text.len());
                let temp = self.content[content_line].text[index1];
                self.content[content_line].text[index1] = self.content[content_line].text[index2];
                self.content[content_line].text[index2] = temp;
            },
            2 => {
                // Add character
                let new_char = rng.gen_range(32, 127); // All printable ASCII characters
                self.content[content_line].text.insert(index1, new_char);
            },
            3 => {
                // Remove character
                if self.content[content_line].text.len() > 1 {
                    // Leave at least one character
                    self.content[content_line].text.remove(index1);
                }
            },
            4 => {
                // You can think of more operations: shift / rotate, mirror, ...
            }
            n => info!("mutate(): unknown operation: {}", n)
        }
    }

    fn calculate_fitness(&mut self) -> f64 {
        let mut constructed_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(120, 70);

        draw_text_line(&mut constructed_img, &self.config.font,
            self.content[0].x as i32, self.content[0].y as i32,
            str::from_utf8(&self.content[0].text).unwrap());

        draw_text_line(&mut constructed_img, &self.config.font,
            self.content[1].x as i32, self.content[1].y as i32,
            str::from_utf8(&self.content[1].text).unwrap());

        root_mean_squared_error(&self.config.original_img, &constructed_img)
    }

    fn reset(&mut self) {
        self.content = vec![
        TextBox{ x: 10, y: 10, text: vec![65, 65, 65, 65, 65, 65, 65, 65, 65] },
        TextBox{ x: 10, y: 40, text: vec![65, 65, 65, 65, 65, 65, 65, 65, 65] }];
    }
}

fn draw_text_line(canvas: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    font: &rusttype::Font, pos_x: i32, pos_y: i32, text: &str) {

    let height: f32 = 18.0;
    let scale = rusttype::Scale { x: height * 1.0, y: height };
    let v_metrics = font.v_metrics(scale);
    let offset = rusttype::point(0.0, v_metrics.ascent);
    let glyphs: Vec<rusttype::PositionedGlyph> = font.layout(text, scale, offset).collect();

    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let x = (x as i32) + bb.min.x + pos_x;
                let y = (y as i32) + bb.min.y + pos_y;
                if x >=0 && y >= 0 && x < canvas.width() as i32 && y < canvas.height() as i32 {
                    canvas.put_pixel(x as u32, y as u32, Luma::<u8>{ data: [(v * 255.0) as u8] } );
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
    let _ = file.read_to_end(&mut font_data).unwrap();

    let collection = rusttype::FontCollection::from_bytes(font_data);
    let font = collection.into_font().unwrap();

    let mut original_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(120, 70);
    draw_text_line(&mut original_img, &font, 10, 10, "Darwin-rs");
    draw_text_line(&mut original_img, &font, 10, 40, "OCR Test!");

    let img_file = Path::new("rendered_text.png");
    let _ = original_img.save(&img_file);

    let ocr_config = OCRConfig { font: font, original_img: original_img };

    let initial_population = make_population(20, &ocr_config);

    let population1 = PopulationBuilder::<OCRItem>::new()
        .set_id(1)
        .initial_population(&initial_population)
        .increasing_exp_mutation_rate(1.01)
        .reset_limit_end(0)
        .finalize().unwrap();

    let population2 = PopulationBuilder::<OCRItem>::new()
        .set_id(2)
        .initial_population(&initial_population)
        .increasing_exp_mutation_rate(1.02)
        .reset_limit_end(0)
        .finalize().unwrap();

    let population3 = PopulationBuilder::<OCRItem>::new()
        .set_id(3)
        .initial_population(&initial_population)
        .increasing_exp_mutation_rate(1.03)
        .reset_limit_end(0)
        .finalize().unwrap();

    let population4 = PopulationBuilder::<OCRItem>::new()
        .set_id(4)
        .initial_population(&initial_population)
        .increasing_exp_mutation_rate(1.04)
        .reset_limit_end(0)
        .finalize().unwrap();

    let population5 = PopulationBuilder::<OCRItem>::new()
        .set_id(5)
        .initial_population(&initial_population)
        .increasing_exp_mutation_rate(1.05)
        .reset_limit_end(0)
        .finalize().unwrap();

    let population6 = PopulationBuilder::<OCRItem>::new()
        .set_id(6)
        .initial_population(&initial_population)
        .increasing_exp_mutation_rate(1.06)
        .reset_limit_end(0)
        .finalize().unwrap();

    let ocr_builder = SimulationBuilder::<OCRItem>::new()
        .fitness(0.0)
        .threads(5)
        .add_population(population1)
        .add_population(population2)
        .add_population(population3)
        .add_population(population4)
        .add_population(population5)
        .add_population(population6)
        .share_fittest()
        .finalize();

    match ocr_builder {
        Err(SimError::EndIterationTooLow) => println!("more than 10 iteratons needed"),
        Ok(mut ocr_simulation) => {
            ocr_simulation.run();

            println!("total run time: {} ms", ocr_simulation.total_time_in_ms);
            println!("improvement factor: {}", ocr_simulation.simulation_result.improvement_factor);
            println!("number of iterations: {}", ocr_simulation.simulation_result.iteration_counter);

            let ref item = ocr_simulation.simulation_result.fittest[0].individual;
            let line1 = str::from_utf8(&item.content[0].text).unwrap();
            let line2 = str::from_utf8(&item.content[1].text).unwrap();
            println!("line1: {}, line2: {}", line1, line2);
        }
    }
}

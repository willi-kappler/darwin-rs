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
use image::{ImageBuffer, Luma};
use imageproc::stats::root_mean_squared_error;
use simplelog::{SimpleLogger, LogLevelFilter};
use std::sync::Mutex;
use std::str;

// internal modules
use darwin_rs::{Individual, SimulationBuilder, PopulationBuilder, SimError};

lazy_static!{
    static ref FONT: rusttype::Font<'static>  = {
        // TODO: use fontconfig-rs in the future: https://github.com/abonander/fontconfig-rs
        let mut file = File::open("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf").unwrap();
        let mut font_data: Vec<u8> = Vec::new();
        let bytes_read = file.read_to_end(&mut font_data).unwrap();
        println!("bytes read: {}", bytes_read);

        let collection = rusttype::FontCollection::from_bytes(font_data);
        let font = collection.into_font().unwrap();
        font
    };
}

lazy_static! {
    static ref ORIGINAL_IMG: Mutex<ImageBuffer<Luma<u8>, Vec<u8>>> = {
        let original_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(640, 70);
        Mutex::new(original_img)
    };
}

#[derive(Debug, Clone)]
struct TextBox {
    x: u32,
    y: u32,
    text: Vec<u8>,
}

#[derive(Debug, Clone)]
struct OCRItem {
    content: Vec<TextBox>,
}

impl Individual for OCRItem {
    fn new() -> OCRItem {
        OCRItem { content: Vec::new() }
    }

    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();

        let operation: u8 = rng.gen_range(0, 6);

        match operation {
            0 => { // Add a new empty text box
                self.content.push( TextBox{ x: 0, y: 0, text: Vec::new() } );
            },
            1 => { // Remove a text box
                if self.content.len() > 1 { // Leave at least one text box.
                    let remove_index = rng.gen_range(0, self.content.len());
                    let _ = self.content.remove(remove_index as usize);
                }
            },
            2 => { // Move the text box around
                if self.content.len() > 0 {
                    let move_index = rng.gen_range(0, self.content.len());
                    let canvas = ORIGINAL_IMG.lock().unwrap();
                    let new_x = rng.gen_range(0, canvas.width());
                    let new_y = rng.gen_range(0, canvas.height());

                    self.content[move_index].x = new_x;
                    self.content[move_index].y = new_y;
                }
            },
            3 => { // Add a random character to the text box
                if self.content.len() > 0 {
                    let content_index = rng.gen_range(0, self.content.len());
                    let add_char_index = if self.content[content_index].text.is_empty() { 0 }
                        else { rng.gen_range(0, self.content[content_index].text.len()) };
                    let random_ascii_value: u8 = rng.gen_range(33, 127);

                    self.content[content_index].text.insert(add_char_index, random_ascii_value);
                }
            },
            4 => { // Remove a character from the text box
                if self.content.len() > 0 {
                    let content_index = rng.gen_range(0, self.content.len());
                    if self.content[content_index].text.len() > 1 { // Leave at least one char
                        let remove_char_index = rng.gen_range(0, self.content[content_index].text.len());
                        self.content[content_index].text.remove(remove_char_index);

                    }
                }
            },
            5 => { // Swap two characters inside a text box
                if self.content.len() > 0 {
                    let content_index = rng.gen_range(0, self.content.len());
                    if self.content[content_index].text.len() > 1 { // Need at least two chars to swap
                        let index1: usize = rng.gen_range(0, self.content[content_index].text.len());
                        let index2: usize = rng.gen_range(0, self.content[content_index].text.len());
                        let tmp_char = self.content[content_index].text[index1];
                        self.content[content_index].text[index1] = self.content[content_index].text[index2];
                        self.content[content_index].text[index2] = tmp_char;
                    }
                }
            },
            _ => println!("unknown operation: {}", operation),
        }
    }

    fn calculate_fitness(&self) -> f64 {
        let mut constructed_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(640, 70);

        for text_box in self.content.iter() {
            match str::from_utf8(&text_box.text) {
                Ok(text) => draw_text_line(&mut constructed_img, &FONT, text_box.x as i32,
                    text_box.y as i32, text),
                _ => {} // Do nothing for now
            }
        }

        let canvas = ORIGINAL_IMG.lock().unwrap();

        root_mean_squared_error(&*canvas, &constructed_img)
    }
}

fn draw_text_line(canvas: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
                  font: &rusttype::Font,
                  pos_x: i32,
                  pos_y: i32,
                  text: &str) {

    // println!("text: {}", text);

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

    {
        let mut canvas = ORIGINAL_IMG.lock().unwrap();

        draw_text_line(&mut canvas, &FONT, 10, 10, "This is a test text!");
        draw_text_line(&mut canvas, &FONT, 10, 40, "Just to see how good OCR works...");

        let img_file = Path::new("rendered_text.png");
        let _ = canvas.save(&img_file);
    }

    let population1 = PopulationBuilder::<OCRItem>::new()
        .set_id(1)
        .individuals(30)
        .increasing_exp_mutation_rate(1.1)
        .reset_limit_increment(100)
        .reset_limit_start(100)
        .reset_limit_end(1000)
        .finalize().unwrap();

    let population2 = PopulationBuilder::<OCRItem>::new()
        .set_id(2)
        .individuals(30)
        .increasing_exp_mutation_rate(1.15)
        .reset_limit_increment(200)
        .reset_limit_start(100)
        .reset_limit_end(2000)
        .finalize().unwrap();

    let population3 = PopulationBuilder::<OCRItem>::new()
        .set_id(3)
        .individuals(30)
        .increasing_exp_mutation_rate(1.2)
        .reset_limit_increment(300)
        .reset_limit_start(100)
        .reset_limit_end(3000)
        .finalize().unwrap();

    let ocr_builder = SimulationBuilder::<OCRItem>::new()
        .fitness(10.0)
        .threads(2)
        .add_population(population1)
        .add_population(population2)
        .add_population(population3)
        .finalize();

    match ocr_builder {
        Err(SimError::EndIterationTooLow) => println!("more than 10 iteratons needed"),
        Ok(mut ocr_simulation) => {
            ocr_simulation.run();

            println!("total run time: {} ms", ocr_simulation.total_time_in_ms);
            println!("improvement factor: {}", ocr_simulation.simulation_result.improvement_factor);
            println!("number of iterations: {}", ocr_simulation.simulation_result.iteration_counter);

            ocr_simulation.print_fitness();

            for content in ocr_simulation.simulation_result.fittest[0].individual.content.iter() {
                println!("x: {}, y: {}, text: {}", content.x, content.y, str::from_utf8(&content.text).unwrap())
            }
        }
    }
}

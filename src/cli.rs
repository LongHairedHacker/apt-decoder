use indicatif::{ProgressBar, ProgressStyle};

use decoder;

const STEPS: u64 = 100;

pub fn decode(input_path: &str, output_path: &str) {
    println!("Decoding {} to {}", input_path, output_path);

    let bar = ProgressBar::new(STEPS).with_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar}] {percent}% ({eta})")
            .unwrap()
            .progress_chars("=> "),
    );
    let res = decoder::decode(input_path, output_path, |progress, _| {
        bar.set_position((progress * STEPS as f32) as u64);
        (true, STEPS as u32)
    });
    bar.finish();

    if let Err(error) = res {
        println!("Unable to decode file: {}", error);
    } else {
        println!("Done!")
    }
}

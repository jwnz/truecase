extern crate clap;
extern crate failure;
extern crate serde_json;

extern crate truecase;

use std::fs::File;
use std::io::Write;

use truecase::ModelTrainer;
use failure::Error;
use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("truecase.rs")
        .version("0.1")
        .author("Aleksei Voronov <despawn@gmail.com>")
        .about("Train a truecasing model, or use one to truecase a sentence.")
        .subcommand(
            SubCommand::with_name("train")
                .about("Create a truecasing model based on training data")
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("FILE")
                        .help("File where the newly trained model will be written.")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .value_name("FILE")
                        .help("File containing training data, one sentence per line.")
                        .takes_value(true)
                        .required(true)
                        .multiple(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("train") {
        // both .unwraps are safe because the arguments are required
        let output_filename = matches.value_of("output").unwrap();
        let input_filenames: Vec<_> = matches.values_of("input").unwrap().collect();
        do_train(input_filenames, output_filename).unwrap(); // FIXME
    }
}

fn do_train(training_filenames: Vec<&str>, model_filename: &str) -> Result<(), Error> {
    let mut trainer = ModelTrainer::new();
    for filename in training_filenames {
        trainer.add_sentences_from_file(filename)?;
    }
    let model = trainer.into_model();
    let serialized = serde_json::to_string(&model)?;
    File::create(model_filename)?.write_all(serialized.as_bytes())?;
    Ok(())
}

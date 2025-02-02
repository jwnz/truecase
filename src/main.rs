use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

use anyhow::{Context, Result};
use clap::{App, Arg, SubCommand};
use truecase::{Model, ModelTrainer};

fn main() -> Result<()> {
    let matches = App::new("truecase.rs")
        .version("0.3")
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
                        .help("File containing training data, one sentence per line. stdin by default")
                        .takes_value(true)
                        .multiple(true),
                ),
        )
        .subcommand(SubCommand::with_name("truecase")
                .about("Create a truecasing model based on training data")
                .arg(
                    Arg::with_name("model")
                        .short("m")
                        .long("model")
                        .value_name("FILE")
                        .help("File containing the truecasing model produced by `train` command")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .value_name("FILE")
                        .help("File containing sentences that need to be truecased, one sentence per line. stdin by default.")
                        .takes_value(true)
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("FILE")
                        .help("File into which truecased sentences will be written")
                        .takes_value(true)
                )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("train") {
        // .unwrap is safe because the argument is required
        let output_filename = matches.value_of("output").unwrap();
        let input_filenames = matches.values_of("input");

        do_train(input_filenames, output_filename)?;
    }

    if let Some(matches) = matches.subcommand_matches("truecase") {
        // .unwrap is safe because the argument is required
        let model_filename = matches.value_of("model").unwrap();
        let input_filename = matches.value_of("input");
        let output_filename = matches.value_of("output");

        do_truecase(model_filename, input_filename, output_filename)?;
    }

    Ok(())
}

fn do_train(training_filenames: Option<clap::Values>, model_filename: &str) -> Result<()> {
    let mut trainer = ModelTrainer::new();

    match training_filenames {
        Some(filenames) => {
            for filename in filenames {
                trainer
                    .add_sentences_from_file(filename)
                    .context(format!("Couldn't load sentences from {}", filename))?;
            }
        }
        None => {
            let stdin_reader = BufReader::new(stdin());
            for sentence in stdin_reader.lines() {
                trainer.add_sentence(&sentence?);
            }
        }
    }

    let model = trainer.into_model();
    model
        .save_to_file(model_filename)
        .context(format!("Couldn't save model into {}", model_filename))?;

    Ok(())
}

fn do_truecase(
    model_filename: &str,
    input_filename: Option<&str>,
    output_filename: Option<&str>,
) -> Result<()> {
    let model = Model::load_from_file(model_filename)
        .context(format!("Couldn't load model from {}", model_filename))?;

    let input: Box<dyn BufRead> = match input_filename {
        Some(filename) => Box::new(BufReader::new(
            File::open(filename).context(format!("Couldn't open input file {}", filename))?,
        )),
        None => Box::new(BufReader::new(stdin())),
    };

    let mut output: Box<dyn Write> = match output_filename {
        Some(filename) => Box::new(
            File::create(filename).context(format!("Couldn't create output file {}", filename))?,
        ),
        None => Box::new(stdout()),
    };

    for sentence in input.lines() {
        let truecased = model.truecase(&sentence?);
        output.write_all(truecased.as_bytes())?;
        output.write_all(b"\n")?;
    }

    Ok(())
}

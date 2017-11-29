extern crate conllx;
extern crate getopts;
extern crate stdinout;
extern crate xml;

use std::env::args;
use std::io::{BufReader, BufWriter, Read};
use std::process;

use conllx::{Features, Sentence, TokenBuilder, WriteSentence};
use getopts::Options;
use stdinout::{Input, OrExit, Output};
use xml::attribute::OwnedAttribute;
use xml::reader;
use xml::reader::{EventReader, XmlEvent};

struct SentenceIter<R>
where
    R: Read,
{
    reader: EventReader<R>,
}

impl<R> SentenceIter<R>
where
    R: Read,
{
    fn new(reader: EventReader<R>) -> Self {
        SentenceIter { reader: reader }
    }
}

impl<R> Iterator for SentenceIter<R>
where
    R: Read,
{
    type Item = reader::Result<Sentence>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tokens = Vec::new();

        let mut features = "bdc:f|bpc:f".to_owned();

        loop {
            let event = match self.reader.next() {
                Ok(event) => event,
                Err(err) => return Some(Err(err)),
            };

            match event {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => if name.local_name == "s" {
                    let bdc = find_attribute_or(&attributes, "bdc", "f");
                    let bpc = find_attribute_or(&attributes, "bpc", "f");
                    features = format!("bdc:{}|bpc:{}", bdc, bpc);
                },
                XmlEvent::EndElement { name } => if name.local_name == "s" {
                    return Some(Ok(Sentence::new(tokens)));
                },
                XmlEvent::Characters(s) => {
                    for token in s.trim().split("\n") {
                        // XXX: throw error when there is no field at all?
                        if let Some(form) = token.split("\t").nth(0) {
                            tokens.push(
                                TokenBuilder::new(form)
                                    .features(Features::from_string(features.clone()))
                                    .token(),
                            );
                        }
                    }
                }
                XmlEvent::EndDocument => return None,
                _ => {}
            }
        }
    }
}

fn find_attribute_or(attrs: &[OwnedAttribute], name: &str, default: &str) -> String {
    attrs
        .iter()
        .find(|a| a.name.local_name == name)
        .map(|a| a.value.clone())
        .unwrap_or(default.to_owned())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] [INPUT_FILE] [OUTPUT_FILE]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    let matches = opts.parse(&args[1..])
        .or_exit("Could not parse arguments", 1);

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.free.len() > 2 {
        print_usage(&program, opts);
        process::exit(1);
    }

    let input = Input::from(matches.free.get(0).map(String::as_str));
    let reader = BufReader::new(input.buf_read().or_exit("Cannot open input for reading", 1));

    let output = Output::from(matches.free.get(1).map(String::as_str));
    let mut writer = conllx::Writer::new(BufWriter::new(
        output.write().or_exit("Cannot open output for writing", 1),
    ));

    for sentence in SentenceIter::new(EventReader::new(reader)) {
        let sentence = sentence.or_exit("Error reading sentence", 1);
        writer
            .write_sentence(&sentence)
            .or_exit("Error writing sentence", 1);
    }
}

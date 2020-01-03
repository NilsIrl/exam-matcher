// TODO: input and output argument could take .multiple(true)
// TODO: questions spanning multiple pages
// TODO: convert files to a format amazon textract can understand if needed

#[macro_use]
extern crate lazy_static;

#[derive(Clone, PartialEq, Debug)]
enum TextType {
    Numeric,
    Alphabetic,
    Undefined,
}

// Returns true if the string passed is identified as a question or multiple choice possibility
fn question_type(text: &str) -> TextType {
    lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r"^\(?(\S{1,4})\s?(\)|\.|\-).*").unwrap(); // There is a capture group around [[:alnum:]] to be used to identity the question vs answer
    };
    match RE.captures(text.trim()) {
        Some(captures) => match captures.get(1).unwrap().as_str().chars().next() {
            Some(first_character) => {
                if first_character.is_numeric() {
                    TextType::Numeric
                } else if first_character.is_alphabetic() {
                    TextType::Alphabetic
                } else {
                    TextType::Undefined
                }
            }
            None => unreachable!(),
        },
        None => TextType::Undefined,
    }
}

use leptess::{leptonica, tesseract};

/// Help!!!
#[derive(structopt::StructOpt)]
#[structopt(version = "0.0.1", author = "Nils André <nils@nilsand.re>")]
struct Opts {
    input_file: String,
    #[structopt(long = "lang", default_value = "eng")]
    language: String,
    output_file: String,
}

fn main() {
    use structopt::StructOpt;
    let opts = Opts::from_args();

    let pix = leptonica::Pix::from_path(std::path::Path::new(&opts.input_file)).unwrap();
    let api = tesseract::TessBaseApiUnitialized::new()
        .init_with_lang(&opts.language)
        .set_image(&pix);

    let mut top_level_text_type = TextType::Undefined;
    let mut previous_text_type = TextType::Undefined;
    let mut top_level_box = None;
    let mut previous_box = None;
    let mut question_number = 1;
    let mut answer_number = 1;

    let boxes = api.get_textlines(true);
    for i in 0..boxes.len() {
        let bbox = std::rc::Rc::new(boxes.get(i));
        api.set_rectangle(&bbox);
        let text = api.get_text();
        let text_type = question_type(&text);
        print!("found {:?}:{}", text_type, text);
        if let TextType::Undefined = top_level_text_type {
            top_level_text_type = text_type.clone();
            previous_text_type = text_type;
            top_level_box = Some(std::rc::Rc::clone(&bbox));
            previous_box = Some(std::rc::Rc::clone(&bbox));
            continue;
        }

        match text_type {
            TextType::Undefined => continue,
            // End of answer or question
            _ => {
                if text_type == top_level_text_type {
                    // New Question
                    pix.clip(&leptonica::Box::new(
                        1,
                        top_level_box.as_ref().unwrap().y(),
                        pix.w() as i32,
                        bbox.y() - top_level_box.as_ref().unwrap().y(),
                    ))
                    .write(
                        std::path::Path::new(&format!(
                            "{}-full-question-{}.png",
                            opts.output_file, question_number
                        )),
                        leptonica::FileFormat::Png,
                    )
                    .unwrap();
                    top_level_box = Some(std::rc::Rc::clone(&bbox));
                    top_level_text_type = text_type.clone();
                    question_number += 1;
                }
                if text_type != top_level_text_type && text_type != previous_text_type {
                    // First multiple choice just after question
                    pix.clip(&leptonica::Box::new(
                        1,
                        previous_box.as_ref().unwrap().y(),
                        pix.w() as i32,
                        bbox.y() - previous_box.as_ref().unwrap().y(),
                    ))
                    .write(
                        std::path::Path::new(&format!(
                            "{}-question{}.png",
                            opts.output_file, question_number,
                        )),
                        leptonica::FileFormat::Png,
                    )
                    .unwrap();
                    answer_number = 1;
                } else {
                    // Another answer
                    pix.clip(&leptonica::Box::new(
                        1,
                        previous_box.as_ref().unwrap().y(),
                        pix.w() as i32,
                        bbox.y() - previous_box.as_ref().unwrap().y(),
                    ))
                    .write(
                        std::path::Path::new(&format!(
                            "{}-question{}-answer{}.png",
                            opts.output_file, question_number, answer_number
                        )),
                        leptonica::FileFormat::Png,
                    )
                    .unwrap();
                    answer_number += 1;
                }
            }
        }

        if i == boxes.len() - 1 {
            pix.clip(&leptonica::Box::new(
                1,
                top_level_box.as_ref().unwrap().y(),
                pix.w() as i32,
                bbox.y() - top_level_box.as_ref().unwrap().y(),
            ))
            .write(
                std::path::Path::new(&format!(
                    "{}-full-question-{}.png",
                    opts.output_file, question_number
                )),
                leptonica::FileFormat::Png,
            )
            .unwrap();
            pix.clip(&leptonica::Box::new(1, bbox.y(), pix.w() as i32, bbox.h()))
                .write(
                    std::path::Path::new(&format!(
                        "{}-question{}-answer{}.png",
                        opts.output_file, question_number, answer_number
                    )),
                    leptonica::FileFormat::Png,
                )
                .unwrap();
        }
        previous_box = Some(bbox);
        previous_text_type = text_type;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_question() {
        assert!(TextType::Numeric == question_type("   1. Something"));
        assert!(TextType::Numeric == question_type(" 1) Something"));
        assert!(TextType::Numeric == question_type("(1) Something"));
        assert!(TextType::Numeric == question_type("1. Indicar cuál tipo de autómata"));
        assert!(TextType::Alphabetic == question_type("(a) Un autómata finito"));
        assert!(TextType::Undefined == question_type("hello"));
        assert!(TextType::Undefined == question_type("Cada acierto un punto sobre"));
        assert!(TextType::Undefined == question_type("Cada acierto un punto sobre 10"));
    }

    #[test]
    fn other_detect_question() {
        assert!(TextType::Undefined == question_type("descuenta 1/2."));
        assert!(TextType::Undefined == question_type("puntuan."));
        assert!(TextType::Undefined == question_type("vacía. En el diagram"));
    }

    #[test]
    fn detect_dash() {
        assert!(question_type("1 - alsdkfj alsdkfj") == TextType::Numeric);
    }

    #[test]
    fn utf8_characters() {
        assert!(TextType::Undefined == question_type("(¢) ala+ba)\" — aa®(ba)”"));
    }
}

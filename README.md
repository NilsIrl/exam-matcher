# exam-matcher
A tool that return the individual question and their possible answers for a multiple choice exams from any file

# Features

* Supports more than [200 file formats](https://imagemagick.org/script/formats.php).

# Dependencies

* Imagemagick

# TODO

* Consider the use of other OCR engines than textract, possibly tesseract. This
  is especially important considering [AWS Textract only supports
  English](https://aws.amazon.com/textract/faqs/).
* Use an actual image library rather than using imagemagick as a cli tool.
  - Rust bindings to imagemagick
  - Other image libraries

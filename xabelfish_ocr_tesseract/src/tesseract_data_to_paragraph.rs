use std::collections::HashMap;

use rusty_tesseract::Data;

pub struct TesseractParagraph {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub words: Vec<String>,
}

impl TesseractParagraph {
    pub fn words_to_string(&self) -> String {
        self.words.join(" ")
    }

    pub fn from_tesseract_data(data: Vec<Data>) -> Vec<TesseractParagraph> {
        let mut paragraphs: HashMap<i32, TesseractParagraph> = HashMap::new();

        for line in data {
            if line.level == 3 {
                // Paragraph
                paragraphs.insert(
                    line.par_num,
                    TesseractParagraph {
                        left: line.left,
                        top: line.top,
                        width: line.width,
                        height: line.height,
                        words: vec![],
                    },
                );
            } else if line.level == 5 {
                // Word
                paragraphs
                    .get_mut(&line.par_num)
                    .unwrap()
                    .words
                    .push(line.text);
            }
        }

        let mut pairs: Vec<(i32, TesseractParagraph)> = paragraphs.into_iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));

        pairs.into_iter().map(|i| i.1).collect()
    }
}

pub struct Language {
    pub code: String,
    pub name: String
}

pub trait Translator {
    fn get_supported_languages() -> Vec<Language>;
    fn has_source_language_auto_detection() -> bool;
    fn translate(image_filename: &str, source_language: &str, target_language: &str);
}
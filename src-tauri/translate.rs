use std::process::Output;

pub mod deepl;

pub struct Language {
    pub code: String,
    pub name: String
}

pub trait Translator {
    fn get_supported_source_languages() -> Vec<Language>;
    fn get_supported_target_languages() -> Vec<Language>;
    fn has_source_language_auto_detection() -> bool;
    async fn translate(&self, text: impl ToString, source_language: Option<impl ToString>, target_language: impl ToString) -> impl ToString;
}
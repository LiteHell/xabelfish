use deepl::{DeepLApi, Lang};

use crate::{config::get_xabelfish_config, translate::Translator};

pub struct DeepLTranslator {

}

impl DeepLTranslator {
    pub fn new () -> Self {
        Self {}
    }
}

impl Translator for DeepLTranslator {
    fn get_supported_source_languages() -> Vec<super::Language> {
        todo!()
    }
    
    fn get_supported_target_languages() -> Vec<super::Language> {
        todo!()
    }

    fn has_source_language_auto_detection() -> bool {
        todo!()
    }
    
    async fn translate(&self, text: impl ToString, source_language: Option<impl ToString>, target_language: impl ToString) -> String {
        let client = DeepLApi::with(&get_xabelfish_config().deepl_api_key).new();
        let result = client.translate_text(text, Lang::KO).await.unwrap();
        result.translations.iter().map(|x| x.text.clone()).reduce( |acc, i| acc + i.as_str()).unwrap_or(String::new())
    }
}
use std::{env::current_exe, path::PathBuf};

use xabelfish_config::{XabelFishEngineConfig, ocr::OcrType, translator::TranslatorType};

fn executable_base_dir() -> PathBuf {
    let mut exe_path = current_exe().unwrap();
    exe_path.pop();
    exe_path
}

fn get_executable(name: PathBuf) -> PathBuf {
    executable_base_dir().join(name)
}

pub fn get_ocr() -> PathBuf {
    get_ocr_of(XabelFishEngineConfig::get_config().ocr_type)
}

pub fn get_translator() -> PathBuf {
    get_translator_of(XabelFishEngineConfig::get_config().translator_type)
}

pub fn get_cont_capture() -> PathBuf {
    return get_executable(PathBuf::from("xabelfish_cont_capture"));
}

fn get_ocr_of(ocr_type: OcrType) -> PathBuf {
    return get_executable(PathBuf::from(match ocr_type {
        OcrType::Tesseract => "xabelfish_ocr_tesseract",
    }));
}

fn get_translator_of(translator_type: TranslatorType) -> PathBuf {
    return get_executable(PathBuf::from(match translator_type {
        TranslatorType::DeepL => "xabelfish_translate_deepl",
    }));
}

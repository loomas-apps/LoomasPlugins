use loomasplugins_model_cache::ModelFile;

/// One model: its name and its file of the pinned commit.
macro_rules! model {
    ($name:literal, $file:literal, $sha256:literal, $bytes:literal) => {
        (
            $name,
            ModelFile {
                file: $file,
                url: concat!(
                    "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/",
                    $file
                ),
                sha256: $sha256,
                bytes: $bytes,
            },
        )
    };
}

/// The whisper.cpp models of OpenAI's Whisper (MIT), as ggml files from `ggerganov/whisper.cpp` on
/// Hugging Face, pinned to one commit so the checksums hold. `.en` models are English only.
pub const MODELS: [(&str, ModelFile); 10] = [
    model!(
        "tiny",
        "ggml-tiny.bin",
        "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        77_691_713
    ),
    model!(
        "tiny.en",
        "ggml-tiny.en.bin",
        "921e4cf8686fdd993dcd081a5da5b6c365bfde1162e72b08d75ac75289920b1f",
        77_704_715
    ),
    model!(
        "base",
        "ggml-base.bin",
        "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        147_951_465
    ),
    model!(
        "base.en",
        "ggml-base.en.bin",
        "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002",
        147_964_211
    ),
    model!(
        "small",
        "ggml-small.bin",
        "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        487_601_967
    ),
    model!(
        "small.en",
        "ggml-small.en.bin",
        "c6138d6d58ecc8322097e0f987c32f1be8bb0a18532a3f88f734d1bbf9c41e5d",
        487_614_201
    ),
    model!(
        "medium",
        "ggml-medium.bin",
        "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        1_533_763_059
    ),
    model!(
        "medium.en",
        "ggml-medium.en.bin",
        "cc37e93478338ec7700281a7ac30a10128929eb8f427dda2e865faa8f6da4356",
        1_533_774_781
    ),
    model!(
        "large-v3",
        "ggml-large-v3.bin",
        "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
        3_095_033_483
    ),
    model!(
        "large-v3-turbo",
        "ggml-large-v3-turbo.bin",
        "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
        1_624_555_275
    ),
];

/// The model file of a model name, or `None` when there is no such model.
pub fn model_named(name: &str) -> Option<&'static ModelFile> {
    MODELS
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, file)| file)
}

/// The model names, for a message that lists them.
pub fn model_names() -> Vec<&'static str> {
    MODELS.iter().map(|(name, _)| *name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_has_a_checksum_and_the_address_of_its_own_file() {
        for (_, file) in &MODELS {
            assert_eq!(file.sha256.len(), 64);
            assert!(file.url.ends_with(file.file), "{}", file.file);
        }
        assert_eq!(model_named("base.en").unwrap().file, "ggml-base.en.bin");
        assert!(model_named("huge").is_none());
    }
}

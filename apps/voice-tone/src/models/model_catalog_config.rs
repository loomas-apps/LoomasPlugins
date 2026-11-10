use loomasplugins_model_cache::ModelFile;

/// A wav2vec2 base model fine-tuned to tell 8 emotions in speech (prithivMLmods
/// Speech-Emotion-Classification, Apache-2.0), as the 8-bit ONNX file of onnx-community. It takes
/// raw sound at 16 kHz, each call's sound scaled to zero mean and unit variance, and gives one
/// score for each emotion.
pub const EMOTION: ModelFile = ModelFile {
    file: "speech-emotion-classification-int8.onnx",
    url: "https://huggingface.co/onnx-community/Speech-Emotion-Classification-ONNX/resolve/main/onnx/model_quantized.onnx",
    sha256: "3217e95e4b28d53913fbe94cd9650b520f8e7ecf2bd643b9e45fc5b8b93b8aab",
    bytes: 95_389_059,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_model_has_a_checksum() {
        assert_eq!(EMOTION.sha256.len(), 64);
        assert!(EMOTION.url.ends_with(".onnx"));
    }
}

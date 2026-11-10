use loomasplugins_model_cache::ModelFile;

/// pyannote segmentation-3.0 (MIT), as the ONNX file of onnx-community. It takes 10 seconds of raw
/// sound at 16 kHz and gives, for each frame of about 17 ms, the log-probabilities of 7 classes:
/// nobody, one of three local speakers, or two of them at once.
pub const SEGMENTATION: ModelFile = ModelFile {
    file: "pyannote-segmentation-3.0.onnx",
    url: "https://huggingface.co/onnx-community/pyannote-segmentation-3.0/resolve/main/onnx/model.onnx",
    sha256: "057ee564753071c0b09b5b611648b50ac188d50846bff5f01e9f7bbf1591ea25",
    bytes: 5_986_908,
};

/// WeSpeaker ResNet34 trained on VoxCeleb2 (CC-BY-4.0, by the WeSpeaker project), the voice print
/// model pyannote 3.1 also uses. It takes 80 Kaldi filter-bank features a frame and gives 256
/// numbers that are close for the same voice.
pub const VOICE_PRINT: ModelFile = ModelFile {
    file: "wespeaker-voxceleb-resnet34-LM.onnx",
    url: "https://huggingface.co/Wespeaker/wespeaker-voxceleb-resnet34-LM/resolve/main/voxceleb_resnet34_LM.onnx",
    sha256: "7bb2f06e9df17cdf1ef14ee8a15ab08ed28e8d0ef5054ee135741560df2ec068",
    bytes: 26_530_309,
};

/// The name a voice pattern file gives the model its print comes from. Prints of two models
/// cannot be compared, so a pattern of another model is refused.
pub const VOICE_PRINT_MODEL: &str = "wespeaker-voxceleb-resnet34-LM";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_has_a_checksum() {
        for model in [SEGMENTATION, VOICE_PRINT] {
            assert_eq!(model.sha256.len(), 64);
            assert!(model.url.ends_with(".onnx"));
        }
    }
}

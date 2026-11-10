use loomasplugins_model_cache::ModelFile;

/// YuNet (2023, MIT, by Shiqi Yu, from the OpenCV model zoo): a face detector that takes a
/// 640 by 640 picture (blue, green, red, 0 to 255) and gives, at strides 8, 16 and 32, a score,
/// a box and five points (eyes, nose, mouth corners) for each place.
pub const FACES: ModelFile = ModelFile {
    file: "face_detection_yunet_2023mar.onnx",
    url: "https://huggingface.co/opencv/face_detection_yunet/resolve/main/face_detection_yunet_2023mar.onnx",
    sha256: "8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4",
    bytes: 232_589,
};

/// The facial expression model of the OpenCV model zoo (MobileFaceNet trained with Progressive
/// Teacher, Apache-2.0). It takes a face aligned on its five points, 112 by 112, red, green and
/// blue scaled to -1 to 1, and gives a score for each of 7 expressions.
pub const EXPRESSIONS: ModelFile = ModelFile {
    file: "facial_expression_recognition_mobilefacenet_2022july.onnx",
    url: "https://huggingface.co/opencv/facial_expression_recognition/resolve/main/facial_expression_recognition_mobilefacenet_2022july.onnx",
    sha256: "4f61307602fc089ce20488a31d4e4614e3c9753a7d6c41578c854858b183e1a9",
    bytes: 4_791_892,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_has_a_checksum_and_a_url_that_ends_with_its_file() {
        for model in [FACES, EXPRESSIONS] {
            assert_eq!(model.sha256.len(), 64);
            assert!(model.url.ends_with(model.file));
        }
    }
}

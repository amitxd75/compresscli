use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

#[test]
fn test_cli_info_command() {
    let mut cmd = Command::cargo_bin("compresscli").unwrap();
    cmd.arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("CompressCLI version"))
        .stdout(predicate::str::contains("System Information"));
}

#[test]
fn test_cli_presets_list_command() {
    let mut cmd = Command::cargo_bin("compresscli").unwrap();
    cmd.arg("presets")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Presets"))
        .stdout(predicate::str::contains("ultrafast"));
}

#[test]
fn test_cli_sample_image_compression() {
    let sample_png = Path::new("examples/samples/sample.png");
    if !sample_png.exists() {
        return;
    }

    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.webp");

    let mut cmd = Command::cargo_bin("compresscli").unwrap();
    cmd.arg("image")
        .arg(sample_png)
        .arg(&output_path)
        .arg("--quality")
        .arg("80")
        .arg("--overwrite")
        .arg("--no-cache")
        .assert()
        .success()
        .stdout(predicate::str::contains("Image compressed successfully"));

    assert!(output_path.exists());
    assert!(output_path.metadata().unwrap().len() > 0);
}

#[test]
fn test_cli_sample_video_compression() {
    let sample_mp4 = Path::new("examples/samples/sample.mp4");
    if !sample_mp4.exists() {
        return;
    }

    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.mp4");

    let mut cmd = Command::cargo_bin("compresscli").unwrap();
    cmd.arg("video")
        .arg(sample_mp4)
        .arg(&output_path)
        .arg("--preset")
        .arg("ultrafast")
        .arg("--overwrite")
        .arg("--no-cache")
        .assert()
        .success()
        .stdout(predicate::str::contains("Video compressed successfully"));

    assert!(output_path.exists());
    assert!(output_path.metadata().unwrap().len() > 0);
}

#[test]
fn test_cli_sample_batch_compression() {
    let samples_dir = Path::new("examples/samples");
    let sample_mp4 = samples_dir.join("sample.mp4");
    let sample_png = samples_dir.join("sample.png");

    // Skip if samples dir doesn't exist or has no media files
    if !samples_dir.exists() || (!sample_mp4.exists() && !sample_png.exists()) {
        return;
    }

    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("compresscli").unwrap();
    cmd.arg("batch")
        .arg(samples_dir)
        .arg("--videos")
        .arg("--images")
        .arg("--overwrite")
        .arg("--no-cache")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch Processing Complete"))
        .stdout(predicate::str::contains(
            "Total files processed successfully",
        ));

    assert!(
        temp_dir
            .path()
            .join("sample_compressed_medium.mp4")
            .exists()
            || temp_dir.path().join("sample.mp4").exists()
    );
    assert!(
        temp_dir.path().join("sample_compressed.jpg").exists()
            || temp_dir.path().join("sample_compressed.png").exists()
            || temp_dir.path().join("sample.jpg").exists()
    );
}

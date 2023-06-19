#[cfg(feature = "android-support-library")]
fn build_android_support_library() {
    use std::{env, fs, path::PathBuf, process::Command};

    let mut java_src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    java_src_dir.push("src");
    java_src_dir.push("droidplugnext");
    java_src_dir.push("java");

    let mut java_src_gradlew = java_src_dir.clone();
    java_src_gradlew.push(
        #[cfg(target_os = "windows")]
        "gradlew.bat",
        #[cfg(not(target_os = "windows"))]
        "gradlew",
    );

    let mut java_build_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    java_build_dir.pop();
    java_build_dir.pop();
    java_build_dir.pop();
    java_build_dir.push("java");

    let result = Command::new(java_src_gradlew)
        .args(&[
            format!("-PbuildDir={}", java_build_dir.to_str().unwrap()),
            "-p".to_string(),
            java_src_dir.to_str().unwrap().to_string(),
            "assemble".to_string(),
        ])
        .output()
        .expect("Gradle failed");

    java_build_dir.push("outputs");
    java_build_dir.push("aar");
    let profiles = vec!["debug", "release"];
    let mut output = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    for profile in profiles.iter() {
        let filename = format!("droidplug-{}.aar", profile);
        java_build_dir.push(filename.clone());
        output.push(filename);
        fs::copy(java_build_dir.clone(), output.clone()).unwrap();
        java_build_dir.pop();
        output.pop();
    }
}

fn main() {
    #[cfg(feature = "android-support-library")]
    build_android_support_library();
}

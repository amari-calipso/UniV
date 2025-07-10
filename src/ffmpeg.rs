use std::time::Duration;

macro_rules! ffmpeg_version {
    () => {
        "b6.0"
    };
}

macro_rules! base_url {
    () => {
        "https://github.com/eugeneware/ffmpeg-static/releases/download/"
    };
}

pub const DOWNLOAD_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60 * 5));
pub const URL: &str = {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        concat!(base_url!(), ffmpeg_version!(), "/ffmpeg-linux-x64.gz")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        concat!(base_url!(), ffmpeg_version!(), "/ffmpeg-linux-arm64.gz")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        concat!(base_url!(), ffmpeg_version!(), "/ffmpeg-darwin-x64.gz")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        concat!(base_url!(), ffmpeg_version!(), "/ffmpeg-darwin-arm64.gz")
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        concat!(base_url!(), ffmpeg_version!(), "/ffmpeg-win32-x64.gz")
    } else {
        ""
    }
};
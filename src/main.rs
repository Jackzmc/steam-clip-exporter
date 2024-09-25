use std::{env, fmt::Display, fs, path::PathBuf, process::Command};

use dialoguer::theme::ColorfulTheme;
use prost::Message;

#[derive(Clone, PartialEq, ::prost::Message)]
struct ClipPb {
    #[prost(message, required, tag = "1")]
    pub data: ClipPbData,
    #[prost(string, tag = "7")]
    pub name: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
struct ClipPbData {
    #[prost(message, required, tag = "5")]
    pub video: ClipPbDataVideo,
}

#[derive(Clone, PartialEq, ::prost::Message)]
struct ClipPbDataVideo {
    #[prost(string, tag = "1")]
    pub video_dir: String,
}

#[derive(Debug)]
struct Clip {
    path: PathBuf,
    info: ClipPb,
}

impl Display for Clip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info.name)
    }
}

fn main() {
    let userdata = steam_dir().join("userdata");
    let users = userdata
        .read_dir()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let user = if let [user] = &users[..] {
        user.path()
    } else {
        unimplemented!()
    };

    let mut clips = user
        .join("gamerecordings")
        .join("clips")
        .read_dir()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    clips.sort_by_key(|entry| entry.metadata().unwrap().modified().unwrap());

    let clips = clips
        .into_iter()
        .rev()
        .map(|entry| Clip {
            path: entry.path(),
            info: ClipPb::decode(fs::read(entry.path().join("clip.pb")).unwrap().as_slice())
                .unwrap(),
        })
        .collect::<Vec<_>>();

    let choice = dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&clips)
        .default(0)
        .interact()
        .unwrap();

    let clip = &clips[choice];

    let video_dir = clip
        .path
        .join("video")
        .join(&clip.info.data.video.video_dir);

    Command::new("sh")
        .current_dir(&video_dir)
        .args(["-c", "cat init-stream0.m4s chunk-stream0-* > stream0.mp4"])
        .status()
        .unwrap();

    Command::new("sh")
        .current_dir(&video_dir)
        .args(["-c", "cat init-stream1.m4s chunk-stream1-* > stream1.mp4"])
        .status()
        .unwrap();

    Command::new("ffmpeg")
        .args([
            "-i",
            "stream0.mp4",
            "-i",
            "stream1.mp4",
            "-c",
            "copy",
            "../../output.mp4",
        ])
        .current_dir(&video_dir)
        .status()
        .unwrap();

    let output = clip.path.join("output.mp4");
    println!("{}", output.display());
}

fn steam_dir() -> PathBuf {
    let mut path: PathBuf = PathBuf::new();
    path.push(env::var("HOME").expect("HOME not set"));
    path.push(".steam");
    path.push("steam");
    path
}

use std::{env, fmt::Display, fs, path::PathBuf, process::Command};
use std::fs::File;
use std::io::Read;
use dialoguer::theme::ColorfulTheme;
use prost::Message;
use serde::Deserialize;

pub mod steam {
    pub mod webuimessages_gamerecordingfiles {
        include!(concat!(env!("OUT_DIR"), "/_.rs"));
    }
}

use steam::webuimessages_gamerecordingfiles::*;

#[derive(Debug)]
struct Clip {
    path: PathBuf,
    info: CGameRecordingClipFile,
}

impl Display for Clip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info.name.as_deref()
            .or(self.path.file_stem().map(|s| s.to_str().unwrap_or("???")))
            .unwrap_or("???"))
    }
}

fn find_local_userdata_dir() -> PathBuf {
    let userdata = steam_dir().join("userdata");
    let users = userdata
        .read_dir()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // println!("users {:?}", users);
    let user_dir = users.iter().filter(|e| e.file_name() != "anonymous").next().expect("no userdata folder found");
    user_dir.path()
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserLocalConfigStore {
    game_recording: GameRecording
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GameRecording {
    background_record_path: String
}

fn find_recording_dir() -> PathBuf {
    let userdata_dir = find_local_userdata_dir();
    println!("user_data path = {:?}", userdata_dir);
    let config_path = userdata_dir.join("config").join("localconfig.vdf");
    let mut config_file = File::open(config_path).unwrap();
    let mut contents = String::new();
    File::read_to_string(&mut config_file, &mut contents).expect("failed to read localconfig.vdf");

    let kv = keyvalues_serde::from_str::<UserLocalConfigStore>(&contents).expect("failed to parse localconfig.vdf");
    PathBuf::from(kv.game_recording.background_record_path)
}

fn get_clip_index(clips: &[Clip], clip_name: Option<String>) -> usize {
    if let Some(clip_name) = clip_name {
        for (i, clip) in clips.iter().enumerate() {
            if clip.path.file_stem().unwrap().to_string_lossy() == clip_name {
                return i;
            }
        }
        eprintln!("Could not find clip \"{}\"", clip_name);
        std::process::exit(1);
    } else {
        let choice = dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
            .items(&clips)
            .default(0)
            .interact()
            .unwrap();
        choice
    }
}

fn main() {
    let recording_dir = find_recording_dir();
    println!("recording path = {:?}", recording_dir);

    let mut clips = recording_dir
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
            info: CGameRecordingClipFile::decode(
                fs::read(entry.path().join("clip.pb")).unwrap().as_slice(),
            )
            .unwrap(),
        })
        .collect::<Vec<_>>();

    let mut args = std::env::args();
    args.next().unwrap();
    let clip_name = args.next();

    let clip_index = get_clip_index(&clips, clip_name);
    let clip = &clips[clip_index];

    let video_dir = clip.path.join("video").join(
        clip.info
            .timelines
            .first()
            .unwrap()
            .recordings
            .first()
            .unwrap()
            .recording_id
            .as_ref()
            .unwrap(),
    );

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

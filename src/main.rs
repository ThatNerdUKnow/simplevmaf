use std::env;

use crate::cli::Cli;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use libvmaf_rs::{
    libvmaf_sys::VmafLogLevel,
    model::{config::ModelConfig, Model},
    picture::Picture,
    video::Video,
    vmaf::Vmaf,
};
pub mod cli;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let args = Cli::parse();
    let w = args.resolution_w.unwrap_or(1920);
    let h = args.resolution_h.unwrap_or(1080);
    let reference: Video = Video::new(args.ref_path, w, h).unwrap();
    let distorted: Video = Video::new(args.dist_path, w, h).unwrap();

    let num_frames = reference.len();

    let mut model = Model::new(ModelConfig::default(), "vmaf_v0.6.1".to_owned())
    .expect("Can't load vmaf model");
    /*let mut model = Model::load_model(ModelConfig::default(), "./examples/vmaf_v0.6.1.json")
        .expect("Couldn't load vmaf model");*/

    let mut vmaf = Vmaf::new(
        VmafLogLevel::VMAF_LOG_LEVEL_DEBUG,
        num_cpus::get() as u32,
        0,
        0,
    )
    .unwrap();

    vmaf.use_features_from_model(&mut model)
        .expect("Can't load model");

    let style =
        ProgressStyle::with_template("{prefix}: {eta_precise} {wide_bar} [{pos}/{len}]").unwrap();

    let decode_progress = ProgressBar::new(num_frames.try_into().unwrap())
        .with_prefix("Calculating Vmaf Scores")
        .with_style(style.clone());

    let framepairs = reference.into_iter().zip(distorted.into_iter());

    let frame_indicies = framepairs
        .enumerate()
        .map(|(i, (reference, distorted))| {
            let i = i + 1;

            let reference: Picture = reference
                .try_into()
                .expect(&format!("Couldn't get reference frame at index {i}"));
            let distorted: Picture = distorted
                .try_into()
                .expect(&format!("Couldn't get distorted frame at index {i}"));
            vmaf.read_framepair(reference, distorted, i as u32)
                .expect(&format!("Couldn't read framepair at index {i}"));
            decode_progress.inc(1);
            i
        })
        .collect::<Vec<_>>();

    vmaf.flush_framebuffers()
        .expect("Couldn't flush frame buffers");

    decode_progress.finish();

    let scores = frame_indicies.iter().map(|i| {
        let score = vmaf
            .get_score_at_index(&mut model, *i as u32)
            .expect("Couldn't get score");
        score
    });

    let sum: f64 = scores.sum();
    let mean = sum / f64::from(num_frames as u32);
    println!("Pooled score: {mean}");
}

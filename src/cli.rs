use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Path to reference video
    #[arg(short,long)]
    pub ref_path: PathBuf,
    /// Path to distorted (compressed) video
    #[arg(short,long)]
    pub dist_path: PathBuf,

    pub resolution_w: Option<u32>,
    pub resolution_h: Option<u32>,
}

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A tool to learn about disk usage, fast!
#[derive(Parser)]
#[command(name = "dua", version, about)]
pub struct Args {
    /// The amount of threads to use. Defaults to 0, indicates auto
    #[arg(short, long, default_value = "0", value_name = "THREADS")]
    pub threads: usize,

    /// The format with which to print byte counts
    #[arg(short, long, env = "DUA_FORMAT", value_name = "FORMAT")]
    pub format: Option<String>,

    /// Display apparent size instead of disk usage
    #[arg(short = 'A', long, env = "DUA_APPARENT_SIZE")]
    pub apparent_size: bool,

    /// Count hard-linked files each time they are seen
    #[arg(short, long, env = "DUA_COUNT_HARD_LINKS")]
    pub count_hard_links: bool,

    /// If set, we will not cross filesystems or traverse mount points
    #[arg(short, long, env = "DUA_STAY_ON_FILESYSTEM")]
    pub stay_on_filesystem: bool,

    /// One or more absolute directories to ignore. Note that these are not ignored if they are passed as input path
    #[arg(
        short,
        long,
        env = "DUA_IGNORE_DIRS",
        value_name = "IGNORE_DIRS",
        default_values = ["/proc", "/dev", "/sys", "/run"]
    )]
    pub ignore_dirs: Vec<String>,

    /// Write a log file with debug information, including panics
    #[arg(long, env = "DUA_LOG_FILE", value_name = "LOG_FILE")]
    pub log_file: Option<PathBuf>,

    /// One or more input files or directories. If unset, we will use all entries
    #[arg(value_name = "INPUT")]
    pub input: Vec<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Launch the terminal user interface
    #[command(visible_alias = "i")]
    Interactive {
        /// One or more input files or directories
        #[arg(value_name = "INPUT")]
        input: Vec<PathBuf>,
    },

    /// Aggregate the consumed space of one or more directories or files
    #[command(visible_alias = "a")]
    Aggregate {
        /// One or more input files or directories. If unset, we will use all entries
        #[arg(value_name = "INPUT")]
        input: Vec<PathBuf>,
    },

    /// Generate shell completions
    #[command(visible_alias = "c")]
    Completions,

    /// Configuration related commands
    #[command(visible_alias = "cfg")]
    Config,
}

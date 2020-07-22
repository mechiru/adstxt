use adstxt::crawler;
use structopt::StructOpt;
use tokio::time;

use std::{fs, path};

#[derive(StructOpt, Debug)]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u64,
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt, Debug)]
enum Cmd {
    #[structopt(name = "crawle")]
    Crawle {
        /// The file path of the domain list.
        #[structopt(short = "f", long, parse(from_os_str))]
        file: path::PathBuf,
        /// Output directory of the crawl result.
        #[structopt(short = "o", long, parse(from_os_str))]
        out_dir: path::PathBuf,
        /// The chunk size of the domain passed to job when crawling.
        #[structopt(long, default_value = "50")]
        chunk_size: usize,
        /// Timeout milliseconds.
        #[structopt(long, default_value = "1000")]
        timeout: u64,
        /// The maximum number of domains to crawl.
        #[structopt(long)]
        limit: Option<usize>,
    },
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    loggerv::init_with_verbosity(opt.verbose).unwrap();

    match opt.cmd {
        Cmd::Crawle {
            file,
            out_dir,
            chunk_size,
            timeout,
            limit,
        } => {
            if !out_dir.exists() {
                fs::create_dir(out_dir.clone()).unwrap();
            }

            let file = fs::read_to_string(file).unwrap();
            let iter = file.lines();
            let domains = if let Some(limit) = limit {
                iter.take(limit).map(ToOwned::to_owned).collect()
            } else {
                iter.map(ToOwned::to_owned).collect()
            };

            crawler::crawle(
                crawler::Config {
                    chunk_size,
                    out_dir,
                    timeout: time::Duration::from_millis(timeout),
                },
                domains,
            )
            .await
            .unwrap();
        }
    }
}

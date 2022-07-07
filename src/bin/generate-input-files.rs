use clap::Parser;
use std::fs;
use wasm_zkp_challenge::msm;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Output directory to store the generated input vectors.
    #[clap(short, long, value_parser)]
    dir: String,

    /// Count of input vectors to generate.
    #[clap(short, long, value_parser, default_value_t = 10)]
    count: usize,

    /// Number of elements, as a power of two, to include in each input vector.
    #[clap(short, long, value_parser, default_value_t = 12)]
    size: usize,
}

fn main() -> Result<(), msm::Error> {
    let args = Args::parse();

    fs::create_dir_all(&args.dir)?;

    let mut append: bool = false;
    for _ in 0..args.count {
        let (points, scalars) = msm::generate_msm_inputs(1 << args.size);
        msm::serialize_input(&args.dir, &points, &scalars, append)?;
        append = true;
    }
    Ok(())
}

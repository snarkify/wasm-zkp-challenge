use clap::Parser;
use wasm_zkp_challenge::msm;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input directory where the input vector files can be found.
    #[clap(short, long, value_parser)]
    dir: String,
}

fn main() -> Result<(), msm::Error> {
    let args = Args::parse();

    let deserialize_hash = {
        let (points, scalars) = msm::deserialize_input(&args.dir)?;
        (msm::hash(&points)?, msm::hash(&scalars)?)
    };
    println!("Hash of input files: {:?}", deserialize_hash);
    Ok(())
}

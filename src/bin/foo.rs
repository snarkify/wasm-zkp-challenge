use ark_bls12_381::fq::FqConfig;
use ark_ff::MontConfig;

fn main() {
    println!(
        "FqConfig::CAN_USE_NO_CARRY: {}",
        FqConfig::CAN_USE_NO_CARRY_OPT
    );
}

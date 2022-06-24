use ark_bls12_381::{FrParameters, G1Affine, G1Projective};
use ark_ec::{msm, AffineCurve, ProjectiveCurve};
use ark_ff::{fields::BitIteratorLE, FpParameters, PrimeField, UniformRand, Zero};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Error, Write};
use std::time::{Duration, Instant};

pub fn generate_msm_inputs(
    size: usize,
) -> (
    Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) {
    let mut rng = ark_std::test_rng();

    let scalar_vec = (0..size)
        .map(|_| <G1Affine as AffineCurve>::ScalarField::rand(&mut rng).into_repr())
        .collect::<Vec<_>>();

    // Vector of multiples 2^i & G_1, used to precompute the "doubling" portion of double and add.
    let g_multiples = {
        let mut x = G1Projective::prime_subgroup_generator();
        let mut multiples = vec![x];

        // TODO: Don't hardcode that constant.
        for _ in 0..FrParameters::MODULUS_BITS {
            x.double_in_place();
            multiples.push(x);
        }
        G1Projective::batch_normalization_into_affine(&multiples)
    };

    // Generate a number of random multipliers to apply to G_1 to generate a set of random bases.
    let factor_vec = (0..size)
        .map(|_| <G1Affine as AffineCurve>::ScalarField::rand(&mut rng).into_repr())
        .collect::<Vec<_>>();

    // Compute the multiples of G_1 using the precomputed tables of 2^i multiples.
    let point_vec = factor_vec
        .iter()
        .map(|r| {
            let bits = BitIteratorLE::new(r);
            let mut p = G1Projective::zero();
            for (i, b) in bits.enumerate() {
                if b {
                    p.add_assign_mixed(&g_multiples[i]);
                }
            }
            p
        })
        .collect::<Vec<_>>();

    let point_vec = G1Projective::batch_normalization_into_affine(&point_vec);

    (point_vec, scalar_vec)
}

/// Currently using Pippenger's algorithm for multi-scalar multiplication (MSM)
pub fn compute_msm(
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) -> G1Projective {
    msm::VariableBaseMSM::multi_scalar_mul(point_vec.as_slice(), scalar_vec.as_slice())
}

pub fn compute_msm_opt(
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) -> <G1Affine as AffineCurve>::Projective {
    msm::MultiExp::compute_msm_opt(point_vec.as_slice(), scalar_vec.as_slice())
}

pub fn write_to_file(
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
    scalar_file: &str,
) -> Result<(), Error> {
    let mut output2 = File::create(scalar_file)?;
    for scalar in scalar_vec {
        let bits = scalar.to_bits_be();
        let bits = bits
            .into_iter()
            .map(|x| {
                if x {
                    format!("{}", 1)
                } else {
                    format!("{}", 0)
                }
            })
            .collect::<Vec<String>>()
            .join("");
        write!(output2, "{}\n", bits)?;
    }
    Ok(())
}

pub fn read_from_file(
    file_loc: &str,
) -> Result<Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>, Error> {
    let mut scalars = Vec::new();
    let input = File::open(file_loc)?;
    let buffered = BufReader::new(input);
    for line in buffered.lines() {
        let mut tmp = Vec::new();
        let line = line?;
        for c in line.chars() {
            if c == '0' {
                tmp.push(false);
            } else {
                tmp.push(true);
            }
        }
        let scalar =
            <<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt::from_bits_be(&tmp);
        scalars.push(scalar);
    }
    Ok(scalars)
}

#[test]
fn test() {
    let K = 16;
    let size = 1 << K;
    let (point_vec, scalar_vec) = generate_msm_inputs(size);
    let _ = write_to_file(scalar_vec.clone(), "./scalar.txt");
    //let scalar = <<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt::from_bits_le(&[true,false]);
    //let scalar_vec1 = read_from_file("./scalar.txt").unwrap();
    let start = Instant::now();
    let res1 = compute_msm(point_vec.clone(), scalar_vec.clone());
    let duration = start.elapsed();
    println!("baseline with size 1<<{}: {:?}", K, duration);
    println!("\n baseline res = {:?}\n", res1.into_affine());

    let start = Instant::now();
    let res2 = compute_msm_opt(point_vec.clone(), scalar_vec.clone());
    let duration = start.elapsed();
    println!("msm_opt with size 1<<{}: {:?}", K, duration);
    println!("\n msm_opt = {:?}\n", res2.into_affine());
}

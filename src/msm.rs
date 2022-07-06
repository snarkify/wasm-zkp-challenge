use ark_bls12_381::{G1Affine, G1Projective};
use ark_ec::{msm, AffineCurve, ProjectiveCurve};
use ark_ff::{fields::BitIteratorLE, BigInteger, PrimeField, UniformRand, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Write};
use std::fs::File;
use std::io::{BufRead, BufReader};

// Define ScalarField and BigInt type aliases to avoid lengthy fully-qualified names.
type ScalarField = <G1Affine as AffineCurve>::ScalarField;
type BigInt = <ScalarField as PrimeField>::BigInt;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not serialize")]
    SerializationError(#[from] ark_serialize::SerializationError),

    #[error("could not open file")]
    FileOpenError(#[from] std::io::Error),
}

pub fn generate_msm_inputs(
    size: usize,
) -> (
    Vec<G1Affine>,
    Vec<BigInt>
) {
    let mut rng = ark_std::test_rng();

    let scalar_vec = (0..size)
        .map(|_| ScalarField::rand(&mut rng).into_bigint())
        .collect::<Vec<_>>();

    // Vector of multiples 2^i & G_1, used to precompute the "doubling" portion of double and add.
    // TODO(victor): This could be improved by implementing a more optimal fixed base multiplcation
    // routine such as fixed base comb.
    let g_multiples = {
        let mut x = G1Projective::prime_subgroup_generator();
        let mut multiples = vec![x];

        // TODO: Don't hardcode that constant.
        for _ in 0..ScalarField::MODULUS_BIT_SIZE {
            x.double_in_place();
            multiples.push(x);
        }
        G1Projective::batch_normalization_into_affine(&multiples)
    };

    // Generate a number of random multipliers to apply to G_1 to generate a set of random bases.
    let factor_vec = (0..size)
        .map(|_| ScalarField::rand(&mut rng).into_bigint())
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
    point_vec: &[G1Affine],
    scalar_vec: &[BigInt],
) -> G1Projective {
    msm::VariableBase::msm(point_vec, scalar_vec)
}

/// Locally optimized version of the variable base MSM algorithm.
pub fn compute_msm_opt(
    point_vec: &[G1Affine],
    scalar_vec: &[BigInt],
) -> G1Projective {
    msm::MultiExp::compute_msm_opt(point_vec, scalar_vec)
}

pub fn write_scalar_bits_to_file(
    scalar_vec: &[BigInt],
    scalar_file: &str,
) -> Result<(), std::io::Error> {
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

pub fn read_scalar_bits_from_file(
    file_loc: &str,
) -> Result<Vec<BigInt>, std::io::Error> {
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
            BigInt::from_bits_be(&tmp);
        scalars.push(scalar);
    }
    Ok(scalars)
}

/*
pub fn serialize_input(
    dir: &str,
    points: &[Point],
    scalars: &[Scalar],
    append: bool,
) -> Result<(), Error> {
    let points_path = format!("{}{}", dir, "/points");
    let scalars_path = format!("{}{}", dir, "/scalars");
    let (f1, f2) = if append {
        let file1 = File::options()
            .append(true)
            .create(true)
            .open(points_path)?;
        let file2 = File::options()
            .append(true)
            .create(true)
            .open(scalars_path)?;
        (file1, file2)
    } else {
        let file1 = File::create(points_path)?;
        let file2 = File::create(scalars_path)?;
        (file1, file2)
    };
    points.serialize(&f1)?;
    scalars.serialize(&f2)?;
    Ok(())
}

pub fn deserialize_input(dir: &str) -> Result<(Vec<Vec<Point>>, Vec<Vec<Scalar>>), Error> {
    let mut points_result = Vec::new();
    let mut scalars_result = Vec::new();
    let points_path = format!("{}{}", dir, "/points");
    let scalars_path = format!("{}{}", dir, "/scalars");
    let f1 = File::open(points_path)?;
    let f2 = File::open(scalars_path)?;

    loop {
        let points = Vec::<Point>::deserialize(&f1);
        let scalars = Vec::<Scalar>::deserialize(&f2);

        let points = match points {
            Ok(x) => x,
            _ => {
                break;
            }
        };

        let scalars = match scalars {
            Ok(x) => x,
            _ => {
                break;
            }
        };

        points_result.push(points);
        scalars_result.push(scalars);
    }

    Ok((points_result, scalars_result))
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use ark_std::time::Instant;

    // Code snippet for writing scalars to a file.
    //let _ = write_to_file(scalar_vec.clone(), "./scalar.txt");
    //let scalar = <<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt::from_bits_le(&[true,false]);
    //let scalar_vec1 = read_from_file("./scalar.txt").unwrap();

    // Input sizes to use in the tests below.
    const K: usize = 16;
    const SIZE: usize = 1 << K;

    #[test]
    fn baseline_msm_doesnt_panic() {
        let (point_vec, scalar_vec) = generate_msm_inputs(SIZE);
        let start = Instant::now();
        let res1 = compute_msm(point_vec.clone(), scalar_vec.clone());
        let duration = start.elapsed();
        println!("baseline with SIZE 1<<{}: {:?}", K, duration);
        println!("\n baseline res = {:?}\n", res1.into_affine());
    }

    #[test]
    fn optimized_msm_doesnt_panic() {
        let (point_vec, scalar_vec) = generate_msm_inputs(SIZE);
        let start = Instant::now();
        let res2 = compute_msm_opt(&point_vec, scalar_vec.clone());
        let duration = start.elapsed();
        println!("msm_opt with SIZE 1<<{}: {:?}", K, duration);
        println!("\n msm_opt = {:?}\n", res2.into_affine());
    }
}

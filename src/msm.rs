use ark_bls12_381::{G1Affine, G1Projective};
use ark_ec::{msm, AffineCurve, ProjectiveCurve};
use ark_ff::{fields::BitIteratorLE, BigInteger, PrimeField, UniformRand, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Write};
use blake3::Hash;
use bytes::BufMut;
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

pub fn generate_msm_inputs(size: usize) -> (Vec<G1Affine>, Vec<BigInt>) {
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
pub fn compute_msm(point_vec: &[G1Affine], scalar_vec: &[BigInt]) -> G1Projective {
    msm::VariableBase::msm(point_vec, scalar_vec)
}

/// Locally optimized version of the variable base MSM algorithm.
pub fn compute_msm_opt(point_vec: &[G1Affine], scalar_vec: &[BigInt]) -> G1Projective {
    msm::MultiExp::compute_msm_opt(point_vec, scalar_vec)
}

pub fn serialize_input(
    dir: &str,
    points: &[G1Affine],
    scalars: &[BigInt],
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

pub fn deserialize_input(dir: &str) -> Result<(Vec<Vec<G1Affine>>, Vec<Vec<BigInt>>), Error> {
    let mut points_result = Vec::new();
    let mut scalars_result = Vec::new();
    let points_path = format!("{}{}", dir, "/points");
    let scalars_path = format!("{}{}", dir, "/scalars");
    let f1 = File::open(points_path)?;
    let f2 = File::open(scalars_path)?;

    loop {
        let points = Vec::<G1Affine>::deserialize(&f1);
        let scalars = Vec::<BigInt>::deserialize(&f2);

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

pub fn hash<E: CanonicalSerialize>(elements: &[E]) -> Result<Hash, Error> {
    let mut buffer = vec![].writer();
    elements.serialize(&mut buffer)?;
    Ok(blake3::hash(&buffer.into_inner()))
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_std::time::Instant;
    use std::fs;

    // Input sizes to use in the tests below.
    const K: usize = 16;
    const SIZE: usize = 1 << K;
    const TEST_DIRECTORY: &'static str = "./tests/";

    #[test]
    fn baseline_msm_doesnt_panic() {
        let (point_vec, scalar_vec) = generate_msm_inputs(SIZE);
        let start = Instant::now();
        let res1 = compute_msm(&point_vec, &scalar_vec);
        let duration = start.elapsed();
        println!("baseline with SIZE 1<<{}: {:?}", K, duration);
        println!("\n baseline res = {:?}\n", res1.into_affine());
    }

    #[test]
    fn optimized_msm_doesnt_panic() {
        let (point_vec, scalar_vec) = generate_msm_inputs(SIZE);
        let start = Instant::now();
        let res2 = compute_msm_opt(&point_vec, &scalar_vec);
        let duration = start.elapsed();
        println!("msm_opt with SIZE 1<<{}: {:?}", K, duration);
        println!("\n msm_opt = {:?}\n", res2.into_affine());
    }

    #[test]
    fn serialization_derserialization_are_consistent() -> Result<(), Error> {
        fs::create_dir_all(TEST_DIRECTORY)?;

        let serialize_hash = {
            let (points, scalars) = generate_msm_inputs(1 << 6);
            serialize_input(TEST_DIRECTORY, &points, &scalars, false)?;

            (hash(&points)?, hash(&scalars)?)
        };

        let deserialize_hash = {
            let (points, scalars) = deserialize_input(TEST_DIRECTORY)?;
            (hash(&points[0])?, hash(&scalars[0])?)
        };
        assert_eq!(serialize_hash, deserialize_hash);
        Ok(())
    }
}

use ark_std::Zero;
use ark_ec::{AffineCurve, ProjectiveCurve, msm};
use ark_ff::{FpParameters, PrimeField, UniformRand, fields::BitIteratorLE};
use ark_bls12_381::{G1Affine, G1Projective, FrParameters};

pub fn generate_msm_inputs(size: usize)
-> (
    Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>, 
    Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
){
    let mut rng = ark_std::test_rng();

    let scalar_vec = (0..size - 1)
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
    let point_vec = factor_vec.iter()
        .map(|r| {
           let bits = BitIteratorLE::new(r);
           let mut p = G1Projective::zero();
           for (i, b) in bits.enumerate() {
               if b {
                   p.add_assign_mixed(&g_multiples[i]);
               }
           }
           p
        }
    ).collect::<Vec<_>>();

    let point_vec = G1Projective::batch_normalization_into_affine(&point_vec);

    (point_vec, scalar_vec)
}

/// Currently using Pippenger's algorithm for multi-scalar multiplication (MSM)
pub fn compute_msm(
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) -> G1Projective
{
    msm::VariableBaseMSM::multi_scalar_mul(point_vec.as_slice(), scalar_vec.as_slice())
}

#[test]
fn test() {
    let size = 1<<14;
    let (point_vec, scalar_vec) = generate_msm_inputs(size);
    let _res = compute_msm(point_vec, scalar_vec);
    // It appears this test just tests that the code runs, not that it reaches any particular
    // result.
}

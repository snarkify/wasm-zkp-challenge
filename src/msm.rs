use ark_ec::{AffineCurve, ProjectiveCurve, msm};
use ark_ff::{PrimeField, UniformRand, Zero, FpParameters, BigInteger};
use ark_bls12_381::G1Affine;


pub fn generate_msm_inputs(size: usize)
-> (
    Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>, 
    Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
){
    let mut rng = ark_std::test_rng();

    let scalar_vec = (0..size - 1)
        .map(|_| <G1Affine as AffineCurve>::ScalarField::rand(&mut rng).into_repr())
        .collect::<Vec<_>>();
    let point_vec = (0..size)
        .map(|_| <G1Affine as AffineCurve>::Projective::rand(&mut rng))
        .collect::<Vec<_>>();
    let point_vec = <<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::batch_normalization_into_affine(&point_vec);

    (point_vec, scalar_vec)
}

/// Currently using Pippenger's algorithm for multi-scalar multiplication (MSM)
pub fn compute_msm(
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) -> <G1Affine as AffineCurve>::Projective
{
    msm::VariableBaseMSM::multi_scalar_mul(point_vec.as_slice(), scalar_vec.as_slice())
}



pub fn compute_pippenger(
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) -> <G1Affine as AffineCurve>::Projective
{
    msm::VariableBaseMSM::pippenger_mul(point_vec.as_slice(), scalar_vec.as_slice())
}



#[test]
fn test() {
    use ark_ff::BigInteger;
    let size = 1<<5;
    let (point_vec, scalar_vec) = generate_msm_inputs(size);
    let res1 = compute_msm(point_vec.clone(), scalar_vec.clone());
    let res2 = compute_pippenger(point_vec, scalar_vec);
    println!("baseline = {:?}", res1);
    println!("pippenger = {:?}", res2);
}
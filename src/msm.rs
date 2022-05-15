use ark_ec::{AffineCurve, ProjectiveCurve, msm};
use ark_ff::{PrimeField, UniformRand, Zero, FpParameters, BigInteger};
use ark_bls12_381::G1Affine;
use std::time::{Duration,Instant};

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

pub fn compute_msm_affine(
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) -> <G1Affine as AffineCurve>::Projective
{
    let (window_buckets,s) = msm::VariableBaseMSM::multi_scalar_mul_batch_affine(point_vec.as_slice(), scalar_vec.as_slice());
    let start = Instant::now();
    let mut Gs:Vec<_> = ark_std::cfg_into_iter!(window_buckets).map(|pts|{
        let zero =<G1Affine as AffineCurve>::Projective::zero();
        let mut res = zero.clone();
        let mut running_sum = zero;
        pts.into_iter().rev().for_each(|b| {
            let tmp = G1Affine::batch_affine_addition(b).into_projective();
            running_sum += tmp;
            res += &running_sum;
        });
        res
    }).collect();
    let duration = start.elapsed();
    println!("baseline+affine  | compute_msm_affine | batch affine buckets: {:?}", duration);
    let lowest = *Gs.first().unwrap();
    // We're traversing windows from high to low.
    lowest
        + &Gs[1..]
        .iter()
        .rev()
        .fold(<G1Affine as AffineCurve>::Projective::zero(), |mut total, sum_i| {
            total += sum_i;
            for _ in 0..s {
                total.double_in_place();
            }
            total
        })
}

pub fn compute_pippenger(
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) -> <G1Affine as AffineCurve>::Projective
{
    msm::VariableBaseMSM::pippenger_mul(point_vec.as_slice(), scalar_vec.as_slice())
}

pub fn compute_pippenger_affine(
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
) -> <G1Affine as AffineCurve>::Projective {
    let (tmp, s)= msm::VariableBaseMSM::pippenger_batch_affine(point_vec.as_slice(),scalar_vec.as_slice());
    let start = Instant::now();
    let mut Gs:Vec<_> = ark_std::cfg_into_iter!(tmp).map(|pts|{
        G1Affine::batch_affine_addition(pts).into_projective()
    }).collect();
    let duration = start.elapsed();
    println!("pippenger+batch | compute_pippenger_affine | batch_affine_addition {:?}", duration);
    let lowest = *Gs.first().unwrap();
    // We're traversing windows from high to low.
    lowest
        + &Gs[1..]
        .iter()
        .rev()
        .fold(<G1Affine as AffineCurve>::Projective::zero(), |mut total, sum_i| {
            total += sum_i;
            for _ in 0..s {
                total.double_in_place();
            }
            total
        })
}




#[test]
fn test() {
    use ark_ff::BigInteger;
    let size = 1<<14;
    let (point_vec, scalar_vec) = generate_msm_inputs(size);
    //let scalar = <<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt::from_bits_le(&[true,false]);
    let ww = point_vec.clone();
    let res1 = compute_msm(point_vec.clone(), scalar_vec.clone());
    let res2 = compute_pippenger(point_vec.clone(), scalar_vec.clone());
    let res3 = compute_pippenger_affine(point_vec.clone(), scalar_vec.clone());
    let res4 = compute_msm_affine(point_vec.clone(), scalar_vec.clone());
    println!("\n baseline = {:?}\n", res1.into_affine());
    println!("pippenger = {:?}\n", res2.into_affine());
    println!("pippenger-affine= {:?}\n", res3.into_affine());
    println!("msm-affine= {:?}\n", res4.into_affine());
}
use ark_bls12_381::G1Affine;
use ark_ec::{AffineCurve, ProjectiveCurve};
use ark_ff::PrimeField;
use wasm_bindgen::prelude::*;

pub mod msm;

#[wasm_bindgen]
pub struct PointVectorInput {
    // What nonsense is this line?
    point_vec: Vec<<<G1Affine as AffineCurve>::Projective as ProjectiveCurve>::Affine>,
}

#[wasm_bindgen]
impl PointVectorInput {
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize) -> Self {
        let (point_vec, _) = msm::generate_msm_inputs(size);

        Self { point_vec }
    }
}

#[wasm_bindgen]
pub struct ScalarVectorInput {
    scalar_vec: Vec<<<G1Affine as AffineCurve>::ScalarField as PrimeField>::BigInt>,
}

#[wasm_bindgen]
impl ScalarVectorInput {
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize) -> Self {
        let (_, scalar_vec) = msm::generate_msm_inputs(size);

        Self { scalar_vec }
    }
}

#[wasm_bindgen]
pub fn compute_msm(point_vec: PointVectorInput, scalar_vec: ScalarVectorInput) {
    let _res = msm::compute_msm(point_vec.point_vec, scalar_vec.scalar_vec);
}

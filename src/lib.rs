use wasm_bindgen::prelude::*;

use ark_bls12_381::G1Affine;
use ark_ec::{AffineCurve, ProjectiveCurve};
use ark_ff::PrimeField;

#[cfg(feature = "debug")]
use console_error_panic_hook;

// If the console.err panic hook is included, initialize it exactly once.
// init_panic_hook is called at the top of every public function.
fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// A println! style macro to allow output to the JS console.
/// ```ignore
/// crate::console_log!("hello from {}", "rust!");
/// ```
/// Will only have an effect in builds with the `debug` feature enabled.
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        #[cfg(feature = "debug")]
        web_sys::console::log_1(&format_args!($($t)*).to_string().into());
    }
}

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
        init_panic_hook();
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
        init_panic_hook();
        let (_, scalar_vec) = msm::generate_msm_inputs(size);

        Self { scalar_vec }
    }
}

#[wasm_bindgen]
pub fn compute_msm(point_vec: PointVectorInput, scalar_vec: ScalarVectorInput) {
    init_panic_hook();
    let _res = msm::compute_msm(&point_vec.point_vec, &scalar_vec.scalar_vec);
}

#[wasm_bindgen]
pub fn compute_msm_opt(point_vec: PointVectorInput, scalar_vec: ScalarVectorInput) {
    init_panic_hook();
    let _res = msm::compute_msm_opt(&point_vec.point_vec, &scalar_vec.scalar_vec);
}

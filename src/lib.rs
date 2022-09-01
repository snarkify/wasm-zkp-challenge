use wasm_bindgen::prelude::*;

use ark_serialize::CanonicalDeserialize;

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

/* Included from the origonal harness, but currently unused.
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
*/

#[wasm_bindgen]
pub struct InstanceObject {
    points: Vec<msm::G1Affine>,
    scalars: Vec<msm::BigInt>,
}

#[wasm_bindgen]
impl InstanceObject {
    #[wasm_bindgen(method, getter)]
    pub fn length(&self) -> usize {
        self.points.len()
    }
}

#[wasm_bindgen]
pub struct InstanceObjectVector {
    instances: Vec<InstanceObject>,
}

#[wasm_bindgen]
impl InstanceObjectVector {
    #[wasm_bindgen(method, getter)]
    pub fn length(&self) -> usize {
        self.instances.len()
    }

    // Copy the instance to hand off the the JS VM.
    // Note that this copies the full undderlying data, which may be quite large.
    pub fn at(&self, i: usize) -> InstanceObject {
        InstanceObject {
            points: self.instances[i].points.clone(),
            scalars: self.instances[i].scalars.clone(),
        }
    }
}

#[wasm_bindgen]
pub fn deserialize_msm_inputs(data: &[u8]) -> InstanceObjectVector {
    init_panic_hook();
    let instances = Vec::<msm::Instance>::deserialize_unchecked(data).unwrap();
    InstanceObjectVector {
        instances: instances
            .into_iter()
            .map(|i| InstanceObject {
                points: i.points,
                scalars: i.scalars,
            })
            .collect(),
    }
}

#[wasm_bindgen]
pub fn generate_msm_inputs(size: usize) -> InstanceObject {
    init_panic_hook();
    let (points, scalars) = msm::generate_msm_inputs(size);
    InstanceObject { points, scalars }
}

#[wasm_bindgen]
pub fn compute_msm(instance: &InstanceObject) {
    init_panic_hook();
    let _res = msm::compute_msm(&instance.points, &instance.scalars);
}

#[wasm_bindgen]
pub fn compute_msm_opt(instance: &InstanceObject) {
    init_panic_hook();
    let _res = msm::compute_msm_opt::<false, true>(&instance.points, &instance.scalars);
}

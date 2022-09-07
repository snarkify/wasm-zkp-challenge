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

#[wasm_bindgen]
pub struct PointVectorInput {
    point_vec: Vec<msm::G1Affine>,
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
    scalar_vec: Vec<msm::BigInt>,
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

    #[wasm_bindgen(method)]
    pub fn points(&self) -> PointVectorInput {
        PointVectorInput {
            point_vec: self.points.clone(),
        }
    }

    #[wasm_bindgen(method)]
    pub fn scalars(&self) -> ScalarVectorInput {
        ScalarVectorInput {
            scalar_vec: self.scalars.clone(),
        }
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
pub fn compute_msm_baseline(point_vec: &PointVectorInput, scalar_vec: &ScalarVectorInput) {
    init_panic_hook();
    let _res = msm::compute_msm_baseline(&point_vec.point_vec, &scalar_vec.scalar_vec);
}

#[wasm_bindgen]
pub fn compute_msm(point_vec: &PointVectorInput, scalar_vec: &ScalarVectorInput) {
    init_panic_hook();
    let _res = msm::compute_msm::<false, true>(&point_vec.point_vec, &scalar_vec.scalar_vec);
}

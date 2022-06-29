import { compute_msm, compute_msm_opt, PointVectorInput, ScalarVectorInput } from "wasm-prover";

const pre = document.getElementById("wasm-prover");

// compute the median of an array
const median = arr => {
  const mid = Math.floor(arr.length / 2),
    nums = [...arr].sort((a, b) => a - b);
  return arr.length % 2 !== 0 ? nums[mid] : (nums[mid - 1] + nums[mid]) / 2;
};

const sizes = [6];

const MARK_START_INPUT = (size) => `MSM 2^${size}: Start generate input`;
const MARK_STOP_INPUT = (size) => `MSM 2^${size}: Stop generate input`;
const MEASURE_INPUT = (size) => `MSM 2^${size}: Input generation time`;

const MARK_START_MSM = (size) => `MSM 2^${size}: Start calculation`;
const MARK_STOP_MSM = (size) => `MSM 2^${size}: Stop calculation`;
const MEASURE_MSM = (size) => `MSM 2^${size}: Calculation time`;

function wasm_bench_msm() {
  let out_text = "\n";
  for (const size of sizes) {
    const repeat = 10;
    const perf = Array.from(
      { length: repeat },
      (_, i) => {
        // Generating the input itself is actually a rather time consuming operation.
        performance.mark(MARK_START_INPUT(size));
        const point_vec = new PointVectorInput(Math.pow(2, size));
        const scalar_vec = new ScalarVectorInput(Math.pow(2, size));
        performance.mark(MARK_STOP_INPUT(size));
        performance.measure(MEASURE_INPUT(size), MARK_START_INPUT(size), MARK_STOP_INPUT(size));

        // Measure the actual MSM computation.
        performance.mark(MARK_START_MSM(size));
        compute_msm(point_vec, scalar_vec);        
        performance.mark(MARK_STOP_MSM(size));
        performance.measure(MEASURE_MSM(size), MARK_START_MSM(size), MARK_STOP_MSM(size));
      }
    );

    const measures = performance.getEntriesByName(MEASURE_MSM(size), "measure");
    let cur_res = `bench_msm(). input vector length: 2^${size}, median performance: ${median(measures.map(({ duration }) => duration))} ms \n`;
    out_text = out_text.concat(cur_res);
  }
  return out_text;
}

// benchmarking msm
pre.textContent = wasm_bench_msm();

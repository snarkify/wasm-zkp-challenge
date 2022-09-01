import { compute_msm, compute_msm_opt, generate_msm_inputs, deserialize_msm_inputs } from "wasm-prover";

const outputPre = document.getElementById("wasm-prover");
const instanceInput = document.getElementById("instance-file");
const runButton = document.getElementById("run-button");

// compute the median of an array
const median = arr => {
  const mid = Math.floor(arr.length / 2),
    nums = [...arr].sort((a, b) => a - b);
  return arr.length % 2 !== 0 ? nums[mid] : (nums[mid - 1] + nums[mid]) / 2;
};

// Parameters for generated MSM inputs.
const repeat = 10;

const MARK_START_DESERIALIZE = () => `Start deserialize input`;
const MARK_STOP_DESERIALIZE = () => `Stop deserialize input`;
const MEASURE_DESERIALIZE = () => `Input deserialize time`;

const MARK_START_GENERATE = (size) => `MSM 2^${size}: Start generate input`;
const MARK_STOP_GENERATE = (size) => `MSM 2^${size}: Stop generate input`;
const MEASURE_GENERATE = (size) => `MSM 2^${size}: Input generation time`;

const MARK_START_MSM = (size) => `MSM 2^${size}: Start calculation`;
const MARK_STOP_MSM = (size) => `MSM 2^${size}: Stop calculation`;
const MEASURE_MSM = (size) => `MSM 2^${size}: Calculation time`;

function buffer2hex(buffer) {
  const array = Array.from(new Uint8Array(buffer))
  const hexarray = array.map(b => b.toString(16).padStart(2, '0'))
  return hexarray.join('')
}

async function deserialize_file_input() {
  // If there is no file input, return undefined.
  if (instanceInput.files.length == 0) {
    return undefined
  }

  const file = instanceInput.files.item(0)
  const data = await file.arrayBuffer()
  // const hash = await crypto.subtle.digest('SHA-256', data)
  // console.log(`Instance input file ${file.name} is of length ${file.size} and hash: ${buffer2hex(hash)}`)
  // Note that this returns an InstanceObjectVector.

  performance.mark(MARK_START_DESERIALIZE())
  const deserialized = deserialize_msm_inputs(new Uint8Array(data))
  performance.mark(MARK_STOP_DESERIALIZE())
  performance.measure(MEASURE_DESERIALIZE(), MARK_START_DESERIALIZE(), MARK_STOP_DESERIALIZE())

  return deserialized
}

async function load_or_generate_msm_inputs() {
  // First check for a file input and deserialize it if one is provided.
  const deserialized = await deserialize_file_input()
  if (deserialized !== undefined) {
    return deserialized
  }

  // No file was provided, so we should generate new inputs.
  const generated = Array.from({ length: repeat }, () => {
    // Generating the input itself is actually a rather time consuming operation.
    performance.mark(MARK_START_GENERATE(size))
    const instance = generate_msm_inputs(Math.pow(2, size))
    performance.mark(MARK_STOP_GENERATE(size))
    performance.measure(MEASURE_GENERATE(size), MARK_START_GENERATE(size), MARK_STOP_GENERATE(size))
    return instance
  })
  return generated
}

async function wasm_bench_msm() {
  let out_text = "\n";

  // Clear marks and measures previously written.
  performance.clearMarks();
  performance.clearMeasures();

  const instances = await load_or_generate_msm_inputs()
  const size = Math.floor(Math.log2(instances.at(0).length)) // Assume all instances as same size.
  console.log(`Running benchmark with ${instances.length} instances of size 2^${size}`)

  // Note: Using a classic for loop because the Rust object, InstanceObjectVector, does not support
  // the iterator interface.
  for (let j = 0; j < 1; j++) {
    for (let i = 0; i < instances.length; i++) {
      console.log(`Running benchmark with instance ${j}/${i}`)
      const instance = instances.at(i)

      // Measure the actual MSM computation.
      performance.mark(MARK_START_MSM(size));
      compute_msm_opt(instance);     
      performance.mark(MARK_STOP_MSM(size));
      performance.measure(MEASURE_MSM(size), MARK_START_MSM(size), MARK_STOP_MSM(size));
    }
  }
  console.log(`Finished running benchmark`)

  // Extract the performance markers and format the aggregate result from all instances.
  const measures = performance.getEntriesByName(MEASURE_MSM(size), "measure");
  let cur_res = `bench_msm(). input vector length: 2^${size}, median performance: ${median(measures.map(({ duration }) => duration))} ms \n`;
  out_text = out_text.concat(cur_res);

  return out_text;
}

// benchmarking msm
runButton.onclick = async () => {
  outputPre.textContent = `running...`
  outputPre.textContent = await wasm_bench_msm()
}

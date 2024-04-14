import wasmInit, {
  create_frac_range,
  create_task_table,
  get_model_path,
  search_with_task_table,
} from "./pkg/adsorption_search_wasm.js";

const runWasm = async () => {
  // const rustWasm = await wasmInit("./pkg/adsorption_search_wasm_bg.wasm");
  const x_range = create_frac_range(0.0, 1.0);
  console.log(x_range);
};

runWasm();

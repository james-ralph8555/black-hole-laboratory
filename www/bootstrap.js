// Import the initializer and the function you want to run
import init, { run } from './pkg/index.js';

async function main() {
  // Wait for the wasm module to be compiled and initialized
  await init();

  // Now that initialization is complete, it's safe to call our function
  run();
}

main().catch(console.error);

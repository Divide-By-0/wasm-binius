![Binius logo](doc/Logo.png "Binius logo")

# Binius WASM Benchmark

Two runs:

run_keccak_example took 2209.515 seconds
main_js took 401.211 seconds

run_keccak_example took 384.138 seconds
main_js took 376.547 seconds

Locally, this executes on my 10 cores in about 15 seconds, which is pretty close to paper benchmarks. In the browser, there is a slowdown of about 20-150x, without any rayon bindgen wasm multithreading, which we can expect to at most make 8x better.

I'm not entirely sure, but `log_size = 23` and the trace calculation code here: 
```
	// Each round state is 64 rows
	// Each permutation is 24 round states
	for perm_i in 0..1 << (log_size - 11) {
		let i = perm_i << 5;

		// Randomly generate the initial permutation input
		let input: [u64; 25] = rng.gen();
		let output = {
			let mut output = input;
			keccakf(&mut output);
			output
		};
  ...
```

seems to imply to me that 1600 bits are getting hashed 4096 times. Compare this to 1024 bit keccak being about ~151K constraints in circom, which we expect to take about 5 seconds in the browser and under a second on an app. Let's say on average keccak takes 1000 seconds in the browser; so that's about 0.25 seconds per 1600 bits, making unparallelized binius around 8x faster than parallelized circom in the browser, even with no parallelization.

# Binius

Binius is a Rust library implementing a cryptographic *succinct non-interactive argument of knowledge* (SNARK) over towers of binary fields. The techniques are described formally in the paper *[Succinct Arguments over Towers of Binary Fields](https://eprint.iacr.org/2023/1784)*.

This library is a work in progress. It is not yet ready for use, but may be of interest for experimentation.

## Usage

At this stage, the primary interfaces are the unit tests and benchmarks. The benchmarks use the [criterion](https://docs.rs/criterion/0.3.4/criterion/) library.

To run the benchmarks, use the command `cargo bench`. To run the unit tests, use the command `cargo test`.

Binius implements optimizations for certain target architectures. To enable these, export the environment variable

```bash
RUSTFLAGS="-C target-cpu=native"
```

Binius has notable optimizations on Intel processors featuring the [Galois Field New Instructions](https://networkbuilders.intel.com/solutionslibrary/galois-field-new-instructions-gfni-technology-guide) (GFNI) instruction set extension. To determine if your processor supports this feature, run

```bash
rustc --print cfg -C target-cpu=native | grep gfni
```

If the output of the command above is empty, the processor does not support these instructions.

### Examples

There are examples of simple commit-and-prove SNARKs in the `examples` directory. For example, you may run

```bash
cargo run --release --example bitwise_and_proof
```

To print out profiling information, set the environment variable `PROFILE_PRINT_TREE=1`:

```bash
PROFILE_PRINT_TREE=1 cargo run --release --example bitwise_and_proof
```

The environment variable `PROFILE_CSV_FILE` can be set to an output filename to dump profiling data to a CSV file for more detailed analysis.

## Support

This project is under active development. The developers with make breaking changes at will. Any modules that are stabilized will be explicitly documented as such.

We use GitLab's issue system for tracking bugs, features, and other development tasks.

This codebase certainly contains many bugs at this point in its development. *We discourage the production use of this library until future notice.* Any bugs, including those affecting the security of the system, may be filed publicly as an issue.

## Authors

Binius is developed by [Irreducible](https://www.irreducible.com).

## License

Copyright 2023-2024 Ulvetanna Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

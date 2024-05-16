Run `wasm-pack build --target web` from this repo.

Then `python3 -m http.server` and go to localhost:8000 and look in console.


We get the odd error:
```
[INFO]: ⬇️  Installing wasm-bindgen...
[INFO]: Optimizing wasm binaries with `wasm-opt`...
unknown name subsection at 1010699
```

which seems fine [here](https://forum.dfinity.org/t/wasm-module-exceeding-maximum-allowed-functions/12409/18) so we just ignore it for now. Maybe something is getting cut off though, since they had an unreasonable speed increase in that forum as well??
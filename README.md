# GamePigeon Word Bites Solver

[![Netlify Status](https://api.netlify.com/api/v1/badges/c2b82b69-ff56-4ee7-9b1a-500c671a9fd9/deploy-status)](https://app.netlify.com/sites/wordbites/deploys)

Available at https://wordbites.netlify.app

## Build
- `cd solver` and build the WASM binary: `sudo wasm-pack build --target web`
- In the main directory, run `python3 -m http.server` (or any other command to launch a web server in that directory)
- Go to `localhost:8000`!

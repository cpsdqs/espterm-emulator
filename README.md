# ESPTerm Emulator
The ESPTerm Emulator emulates the part of [ESPTerm](https://github.com/ESPTerm/espterm-firmware) that runs on a Wi-Fi chip; replacing it with a regular shell. This is intended for development of the [ESPTerm front end](https://github.com/ESPTerm/espterm-front-end), but most terminal features work well enough.

## Usage
1. Clone this repository
2. Symlink the `out` directory of the `espterm-front-end` repo to `web` in this repo
3. Run `cargo run --release` in the repository root
4. Go to `localhost:3000` in a web browser

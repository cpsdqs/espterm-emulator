# ESPTerm Emulator
The ESPTerm Emulator emulates the part of [ESPTerm](https://github.com/ESPTerm/espterm-firmware) that runs on a Wi-Fi chip; replacing it with a regular shell. This is intended for development of the [ESPTerm front end](https://github.com/ESPTerm/espterm-front-end), but most terminal features work well enough.

## Usage
1. Clone this repository
2. In the repository directory, run `npm install`
  - This might require installing [`cargo`](https://www.rust-lang.org) first; I'm not really sure what [`neon`](https://neon-bindings.com) does
3. Run `node [path to espterm-emulator cloned repository] [path to espterm-front-end/out] [port] [width] [height]`
  - Path to out: required
  - Port: optional, default: 3000
  - Width: optional, default: 80
  - Height: optional, default: 25
4. Go to `localhost:3000` in a web browser

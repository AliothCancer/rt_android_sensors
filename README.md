# Description
The termux_server sends `termux-sensors` stdout through a tcp stream which is captured by laptop-server, which then use `egui_plot` v0.33 to plot realtime.
It is just a demo so every frame of the realtime plot clone all the data sent.

## Workspace description
The cargo workspace contains two cargo projects:
- termux_server:
  - Wrap the `termux-sensors` cmd from the termux-api, support some Message struct to control internal threads through Sender<Message> from the main thread. 
- laptop_server (it's currently only a client):
  - Receive data from termux_server
  - Realtime plot with `egui_plot` v0.33
  - Send messages to termux_server to control it (not implemented)

## termux-server

**target**: `aarch64-linux-android` (termux on android arm64)
- Note that this imply you provide the NDK path inside cargo_workspace/.cargo/config.toml

**Current Behavior**: 
- Running `./termux-server` starts 3 threads:
  - main
  - SensorWorker

# Demo
https://github.com/user-attachments/assets/14acfd0a-143b-48af-8bae-1cfbb375b6b2


cargo build -p termux_server --target aarch64-linux-android --release;
scp -P 8022 ./target/aarch64-linux-android/release/termux_server u0_a279@172.17.62.188:/data/data/com.termux/files/home
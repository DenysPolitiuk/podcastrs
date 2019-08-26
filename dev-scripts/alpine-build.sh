export PKG_CONFIG_ALLOW_CROSS=1
export OPENSSL_INCLUDE_DIR=/usr/include
export OPENSSL_LIB_DIR=/usr/lib
export RUSTFLAGS="-C linker=musl-gcc -C link-arg=-Wl,-rpath=/usr/lib"
cargo build --target x86_64-unknown-linux-musl --release

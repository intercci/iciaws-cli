cargo clean -p iciaws_router
cargo lambda build --release
./run-local.sh

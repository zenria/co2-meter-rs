
@_usage:
    just -l

check_rbp_zero:
    cross check --target arm-unknown-linux-gnueabihf

build_rbp_zero:
    PKG_CONFIG_ALLOW_CROSS=1 cross build --target arm-unknown-linux-gnueabihf --release


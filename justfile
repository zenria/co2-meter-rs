
@_usage:
    just -l

check_rbp_zero:
    cross check --target arm-unknown-linux-gnueabihf

build_rbp_zero:
    cross build --target arm-unknown-linux-gnueabihf --release

deploy target: build_rbp_zero
    scp target/arm-unknown-linux-gnueabihf/release/co2-meter {{target}}:
    ssh {{target}} "sudo service co2-meter stop; sudo cp co2-meter /usr/local/bin && sudo service co2-meter start"
    @echo "Deployed!"


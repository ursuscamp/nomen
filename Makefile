.PHONY: mac-aarch64 linux-amd64 windows-amd64 release

mac-aarch64:
	cargo build --release --target aarch64-apple-darwin

linux-amd64:
	TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl

# Setup: https://gist.github.com/Mefistophell/9787e1b6d2d9441c16d2ac79d6a505e6
windows-amd64:
	TARGET_CC=x86_64-w64-mingw32-gcc cargo build --release --target x86_64-pc-windows-gnu

release: mac-aarch64 linux-amd64 windows-amd64
	mkdir -p release
	zip release/nomen-mac-aarch64-$(VERSION).zip target/aarch64-apple-darwin/release/nomen
	zip release/nomen-linux-amd64-$(VERSION).zip target/x86_64-unknown-linux-musl/release/nomen
	zip release/nomen-windows-amd64-$(VERSION).zip target/x86_64-pc-windows-gnu/release/nomen.exe
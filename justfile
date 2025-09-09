[group('run')]
linux-run:
	cargo run --target x86_64-unknown-linux-gnu 

[group('run')]
[doc("Use device with the given serial (see `adb devices`)")]
android-run DEVICE: install-apk
	cargo apk run --lib -d {{DEVICE}}

[private]
install-apk:
	#!/bin/bash
	if [[ ! -x "$(command -v cargo-apk)" ]]; then
		echo "Install cargo-apk"
		cargo install cargo-apk 
	fi

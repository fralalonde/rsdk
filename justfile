#!just
# Sane make-like command runner: https://github.com/casey/just

# Default target
pre-release: format lint test

# Unit and doc tests
test:
	cargo test

# Release installer tests (POSIX: static + sandboxed integration)
test-installer:
	sh test/test-release-installer-posix.sh
	sh test/test-release-installer-posix-integration.sh

format:
	cargo fmt

lint:
	cargo clippy

clean:
	cargo clean

release-major:
    ./release.sh major

release-minor:
    ./release.sh minor

release-patch:
    ./release.sh patch
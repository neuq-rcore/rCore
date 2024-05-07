.PHONY: clippy clippy-%

clippy: clippy-user clippy-os

clippy-%:
	cd $* && cargo clippy --all-features

# Only run format check in kernel crate Since user crate will be removed in the future
format-check:
	@cd os && cargo fmt --check

%:
	@cd os && make -s $@
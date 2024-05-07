.PHONY: clippy clippy-%

clippy: clippy-user clippy-os

clippy-%:
	cd $* && cargo clippy --all-features

%:
	@cd os && make -s $@
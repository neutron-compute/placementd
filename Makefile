#
# This Makefile helps with development of placementd but is not required
#
include base.mk
.PHONY: build clean develop

### Kubernetes targets
################################################################################

### Rust targets
build: target/debug/placementd ## Build the Rust project

SOURCES=$(shell find src -type f -iname '*.rs')
target/debug/placementd: Cargo.toml $(SOURCES)
	$(CARGO) build

check: Cargo.toml $(SOURCES)
	cargo fmt
	cargo test
################################################################################

### General targets
.PHONY: develop
develop:  ## Set up the development environment
	+$(MAKE) -C $@
	$(foreach POD, \
		$(shell $(KUBECTL) get pod -n placementd -o custom-columns=:metadata.name), \
		$(KUBECTL) exec -n placementd $(POD) -- /bin/sh -c "pkill placementd || true" ; \
		$(KUBECTL) cp -n placementd target/debug/placementd $(POD):/tmp/; )

clean: ## Remove temprary targets
	+$(MAKE) -C develop $@
	+$(MAKE) -C migrations $@
	$(CARGO) clean
################################################################################

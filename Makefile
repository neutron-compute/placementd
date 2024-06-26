#
# This Makefile helps with development of placementd but is not required
#
include base.mk
.PHONY: build clean develop migrations

WEBAPP=target/debug/placementd-web

### Kubernetes targets
################################################################################

### Rust targets
build: $(WEBAPP) ## Build the Rust project

SOURCES=$(shell find . -type f -iname '*.rs')
$(WEBAPP): Cargo.toml $(SOURCES)
	DATABASE_URL=$(DATABASE_URL) $(CARGO) build

check: Cargo.toml $(SOURCES) migrations
	$(CARGO) fmt
	DATABASE_URL=$(DATABASE_URL) $(CARGO) check
	DATABASE_URL=$(DATABASE_URL) $(CARGO) test
################################################################################

### General targets
.PHONY: develop
develop:  ## Set up the development environment
	+$(MAKE) -C $@
	./develop/copy-bins

migrations: ## Run the migrations, must have `DATABASE_URL` set
	+$(MAKE) -C $@
	
clean: ## Remove temprary targets
	+$(MAKE) -C develop $@
	+$(MAKE) -C migrations $@
	$(CARGO) clean
################################################################################

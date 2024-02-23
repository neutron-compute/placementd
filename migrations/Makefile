include ../base.mk
MIGRATIONS:=$(CURDIR)/postgres
SQLX=$(shell which sqlx)
.DEFAULT_GOAL:= migrate
.PHONY: clean depends migrate new

depends: .depends ## Install the dependencies needed for running migrations
.depends:
	@which sqlx || (echo ">> 'sqlx' is not installed, run 'make install-deps'" && exit 1)
	touch -f .depends

install-deps: ## Install development dependendcies
	cargo install sqlx-cli --no-default-features --features sqlite,postgres

migrate: depends $(wildcard $(MIGRATIONS)/*.sql) ## Migrate everything
	sqlx migrate run --source $(MIGRATIONS) --database-url=$(DATABASE_URL)

new: depends ## Create a new migration
	@read -p ">> Enter the migration name: " DESCRIPTION && \
	sqlx migrate add --source $(MIGRATIONS) $$DESCRIPTION

clean: ## Remove temporary files
	rm -f .depends $(DATABASE_FILE)

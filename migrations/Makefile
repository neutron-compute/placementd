include ../base.mk
DATABASE_FILE:=../test.db
DATABASE_URL:=sqlite:$(DATABASE_FILE)
SQLX=$(shell which sqlx)
.DEFAULT_GOAL:= migrate
.PHONY: clean depends migrate

depends: .depends create ## Install the dependencies needed for running migrations
.depends:
	@which sqlx || (echo ">> 'sqlx' is not installed, run 'make install-deps'" && exit 1)
	touch -f .depends

install-deps: ## Install development dependendcies
	cargo install sqlx-cli --no-default-features --features sqlite

migrate: depends ## Migrate everything
	sqlx migrate run --source $(CURDIR) --database-url $(DATABASE_URL)

create: $(DATABASE_FILE) ## Create the test database

$(DATABASE_FILE): .depends
	sqlx db create --database-url $(DATABASE_URL)

new: depends ## Create a new migration
	@read -p ">> Enter the migration name: " DESCRIPTION && \
	sqlx migrate add --source $(CURDIR) $$DESCRIPTION

clean: ## Remove temporary files
	rm -f .depends $(DATABASE_FILE)

include ../base.mk
.PHONY: setup
.DEFAULT_GOAL:=setup
SPECS:=$(wildcard *.yml)

setup: .setup  ## Set up the development services

.setup: $(SPECS)
	$(foreach SPEC, $(SPECS), $(KUBECTL) apply -f $(SPEC);)
	touch .setup

clean: ## Clean up development resources
	$(foreach SPEC, $(SPECS), $(KUBECTL) delete -f $(SPEC) || true;)
	rm -f .setup

#
# This Makefile helps with development of placementd but is not required
#

.DEFAULT_GOAL := help

.PHONY: apply delete
apply: contrib/app.yml ## Apply the test kubernetes configuration
	kubectl apply -f contrib/app.yml 
delete: contrib/app.yml ## Delete the test kubernetes configuration
	kubectl delete -f contrib/app.yml

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

MAKEPATH:=$(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

json-stdin:
	@cd $(MAKEPATH); cat $(MAKEPATH)/whiskers.yaml \
		| yq -o json \
		| butane \
		--pretty \
		--strict \
		| bat -l json

base64:
	@cd $(MAKEPATH); cat $(MAKEPATH)/whiskers.yaml \
		| yq -o json \
		| butane \
		--pretty \
		--strict \
		| base64

yaml-file:
	@bat $(MAKEPATH)/whiskers.yaml
	@cd $(MAKEPATH); butane \
		--pretty \
		--strict \
		< whiskers.yaml \
		| bat -l json

docker-ignition-output:
	@bat $(MAKEPATH)/whiskers.yaml
	-@rm $(MAKEPATH)/whiskers.ign
	@cd $(MAKEPATH); docker run \
		--interactive \
		--rm quay.io/coreos/butane:release \
		--pretty \
		--strict \
		< whiskers.yaml \
		> whiskers.ign
	@bat $(MAKEPATH)/whiskers.ign

node_modules: package.json ## Installs the package's dependencies
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm install
.PHONY: node_modules

release-build: node_modules ## Builds the TypeScript package
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env NODE_ENV=production npm run build
.PHONY: build

build: node_modules ## Builds the TypeScript package
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run build
.PHONY: build

test: node_modules ## Tests the TypeScript package
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run test
.PHONY: test

run: start ## Alias for `make start`
.PHONY: run

start: node_modules ## Runs a dev server for the package/app
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run start
.PHONY: start

watch: start ## Alias for `make start`
.PHONY: watch

clean: ## Cleans all build/test temporary work files
	@echo "--- [$(shell basename ${CURDIR})] $@"
	rm -rf ./node_modules ./dist ./target ./lib

lint: node_modules ## Runs code/style linting
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run eslint
.PHONY: lint


## node_modules: Installs the package's dependencies
default--node_modules: package.json
	$(call header,$@)
	pnpm install
.PHONY: default--node_modules

## clean: Cleans all build/test temporary work files
default--clean:
	$(call header,$@)
	pnpm run clean
	rm -rf ./node_modules
.PHONY: default--clean

## build: Builds the TypeScript package
default--build: node_modules
	$(call header,$@)
	pnpm run build
.PHONY: default--build

## release-build: Builds the TypeScript package for production
default--release-build: node_modules
	$(call header,$@)
	env NODE_ENV=production npm run build
.PHONY: default--release-build

## check-type: Performs TypeScript type checking
default--check-type: node_modules
	$(call header,$@)
	pnpm run build:check
.PHONY: default--check-type

## check-lint: alias for check fmt - as they are the same in our js repos
default--check-lint: node_modules
	$(call header,$@)
	pnpm run fmt:check
.PHONY: default--check-format

## check-format: Checks all code linting/formatting for the TypeScript package
default--check-format: node_modules
	$(call header,$@)
	pnpm run fmt:check
.PHONY: default--check-format

## check: Checks all linting, formatting, & other rules
default--check: node_modules
	$(call header,$@)
	pnpm run check
.PHONY: default--check

## fix-lint: Updates code with linting fixes for the package (may modify sources)
default--fix-lint:
	$(call header,$@)
	pnpm run lint:fix
.PHONY: default--fix-lint

## fix-format: Updates code formatting for the package (may modify sources)
default--fix-format:
	$(call header,$@)
	pnpm run fmt
.PHONY: default--fix-format

## fix: Updates all linting fixes & formatting for the package (may modify sources)
default--fix: fix-format fix-lint
.PHONY: default--fix

## start: Runs a dev server for the package/app
default--start: node_modules
	$(call header,$@)
	pnpm run start
.PHONY: default--start

## run: Alias for `make start`
default--run: start
.PHONY: default--run

## watch: Runs default watch task for the package/app
default--watch: node_modules
	$(call header,$@)
	pnpm run build:watch
.PHONY: default--watch

## test: Tests the TypeScript package
default--test: node_modules
	$(call header,$@)
	pnpm run test
.PHONY: default--test

## prepush: Runs all checks & tests required before pushing commits
default--prepush: check test
.PHONY: default--prepush

# Thanks to:
# https://newbedev.com/make-file-warning-overriding-commands-for-target
%: default--%
	@true

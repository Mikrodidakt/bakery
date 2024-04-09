## help               - Show this help.
.PHONY: help
help:
	@fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'

## build              - Build bakery using cargo
.PHONY: build
build:
	cargo build

## test               - Run all tests using cargo
.PHONY: test
test:
	cargo test

## install            - Install bakery under $HOME/.cargo using cargo
.PHONY: install
install:
	cargo install --path .

## install-deb        - Install latest locally built bakery under /usr/bin using deb package
.PHONY: install-deb
install-deb:
	dpkg -i artifactory/bakery.deb

## deb-package        - Create a debian package from the latest local build
.PHONY: deb-package
deb-package: test
	./scripts/do_deb_package.sh

## setup-rust         - Setup rust on local machine supports debian/ubuntu
.PHONY: setup-rust
setup-rust:
	./scripts/setup-rust.sh

## setup-docker       - Setup docker on local machine supports debian/ubuntu
.PHONY: setup-docker
setup-docker:
	./scripts/setup-docker.sh

## docker-build       - Build a bakery workspace docker image
.PHONY: docker-build
docker-build:
	(./docker/do_docker_build.sh)

## docker-shell       - Open a bakery workspace docker shell
docker-shell:
	(./docker/do_docker_shell.sh)

## release            - Create a release build including a deb package
.PHONY: release
release:
	./scripts/do_release.sh

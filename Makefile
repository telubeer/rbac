default:
	cat Makefile
version:
	@echo $(shell git log -1 --format="%cd" --date=short | sed s/-//g).\
	$(shell git rev-list --count HEAD)_\
	$(shell git rev-parse --short HEAD)
clean-build:
	rm -rf ${PWD}/target/release
clean-archpkg:
	rm -rf .pkgbuild/pkg
	rm -rf .pkgbuild/src
	rm -f .pkgbuild/version.txt
	rm -f .pkgbuild/*.tar.xz
clean: clean-build clean-archpkg
build:
	cargo build --release
build-archpkg:
	make -s version > .pkgbuild/version.txt
	chown -R builder .
	cd .pkgbuild; sudo -u builder makepkg -sc --noconfirm
	ls .pkgbuild/*pkg.tar.xz
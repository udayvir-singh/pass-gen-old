INSTALL_PATH = ${HOME}/.local/bin/pass-gen

ifndef VERBOSE
.SILENT:
endif

# ---------------------- #
#          HELP          #
# ---------------------- #
help:
	echo "Usage: make [TARGETS]"
	echo
	echo "Targets:"
	echo "  build      compile pass-gen using cargo"
	echo "  install    install pass-gen on this system"
	echo "  uninstall  uninstall pass-gen from this system"
	echo
	echo "Example:"
	echo "  make install"


# ---------------------- #
#        INSTALL         #
# ---------------------- #
build:
	echo :: BUILDING PASS-GEN
	cargo build -r

install: build
	install -D -m 0755 target/release/pass-gen $(INSTALL_PATH)
	echo :: INSTALLED PASS-GEN

uninstall:
	rm -rf $(INSTALL_PATH)
	echo :: UNINSTALLED PASS-GEN

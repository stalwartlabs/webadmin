#
# Buid webadmin
#
all:
	@echo No build action,
	@echo Makefile has further information.

# the reason why this file exists
define FURTHER_INFORMATION
#
# Doing `cargo build` will NOT build the `webadmin` you want.
# You need `trunk` from https://trunkrs.dev
# One way to get it is:

cargo install --locked trunk

# To avoid
#   couldn't find application wasm-bindgen (version: m.n.p),
#   unable to download in offline mode

cargo install wasm-bindgen-cli \
  --version $$( grep --after=1 'name = "wasm-bindgen"' Cargo.lock \
	| awk -F= '$$1 ~ /version/ { print $$2}' | tr -d '"' )

# To avoid
#   couldn't find application wasm-opt (version: <any>),
#   unable to download in offline mode

sudo apt install binaryen

#
# In theory you can do now `trunk build --release` or `make it`.
# But `trunk` calls `npx tailwindcss -i input.css -o style/output.css`,
# so you have to be ready for it:

npm install tailwindcss

#
# For what it is worth, I installed `npm` by:

sudo apt install npm

endef

# To get that information on standard out, type `make show`
show:
	$(info $(FURTHER_INFORMATION))

# The actual build, type `make it`
it:
	trunk build --release
	rm -f webadmin.zip # fresh start, avoids updates of the .zip
	cd dist && zip -r ../webadmin.zip *
	# TODO make the build reproducable

install:
	install -m 0755 -d $(DESTDIR)/usr/share/stalwart-webadmin
	install -m 0644 webadmin.zip $(DESTDIR)/usr/share/stalwart-webadmin


update:
	npx update-browserslist-db@latest
	@# Why you should do it
	@#  regularly: https://github.com/browserslist/update-db#readme

# l l

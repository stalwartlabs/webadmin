#
# Buid webadmin
#
all:
	trunk build --release
	cd dist && zip -r webadmin.zip *

install:
	install -m 0755 -d $(DESTDIR)/usr/share/stalwart-webadmin
	install -m 0644 webadmin.zip $(DESTDIR)/usr/share/stalwart-webadmin

# l l

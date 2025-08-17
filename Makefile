#
# Buid webadmin
#
all:
	trunk build --release
	cd dist && zip -r webadmin.zip *

# l l

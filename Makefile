-include config.mk
include misc/mk/defaults.mk
include misc/mk/paths.mk

include boot/$(ARCH)/make.mk
include misc/mk/parts/*.mk

.PHONY: all clean kernel

all: kernel

clean:
	rm -rfd $(BUILD_DIR)

kernel: $(KERNEL_BIN)

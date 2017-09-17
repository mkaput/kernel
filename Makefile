.DEFAULT_GOAL := all

-include config.mk
include misc/mk/defaults.mk
include misc/mk/paths.mk

include boot/$(TARGET)/make.mk
include misc/mk/parts/*.mk

.PHONY: all clean kernel

all: kernel

kernel: $(KERNEL_BIN)

clean:
	rm -rfd $(BUILD_DIR)

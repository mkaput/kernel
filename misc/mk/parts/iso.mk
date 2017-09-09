.PHONY: iso clean-iso

iso: $(PACKAGE_ISO)

clean-iso:
	rm -rfd $(shell dirname $(PACKAGE_ISO))

$(PACKAGE_ISO): $(KERNEL_BIN) $(GRUB_CFG_DIR)/**
	mkdir -p $(@D)/_contents/boot
	cp $(KERNEL_BIN) $(@D)/_contents/boot/kernel.bin
	cp -r $(GRUB_CFG_DIR) $(@D)/_contents/boot
	$(GRUB_MKRESCUE) -o $(PACKAGE_ISO) $(@D)/_contents


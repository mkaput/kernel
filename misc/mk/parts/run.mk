.PHONY: run

run: $(PACKAGE_ISO)
	$(QEMU_RUN) -cdrom $(PACKAGE_ISO)

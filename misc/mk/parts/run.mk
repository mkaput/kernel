.PHONY: run rundbg

run: $(PACKAGE_ISO)
	$(QEMU_RUN) -cdrom $(PACKAGE_ISO)

rundbg: $(PACKAGE_ISO)
	$(QEMU_RUN) -s -S -cdrom $(PACKAGE_ISO)

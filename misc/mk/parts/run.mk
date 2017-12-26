.PHONY: run rundbg

run: $(PACKAGE_ISO)
	$(QEMU_RUN) -cdrom $(PACKAGE_ISO)

rundbg: $(PACKAGE_ISO)
	$(QEMU_RUN) -s -S -cdrom $(PACKAGE_ISO)

rundbg_bootloop: $(PACKAGE_ISO)
	$(QEMU_RUN) -s -d int -no-reboot -cdrom $(PACKAGE_ISO)

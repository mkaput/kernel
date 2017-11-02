.PHONY: book-build book-serve book-pdf

book-build:
	mkdir -p $(TARGET_BOOK_BUILD)/www
	$(GITBOOK) build $(ROOT_DIR) $(TARGET_BOOK_BUILD)/www

book-serve:
	mkdir -p $(TARGET_BOOK_BUILD)/www
	$(GITBOOK) serve $(ROOT_DIR) $(TARGET_BOOK_BUILD)/www

book-pdf:
	mkdir -p $(TARGET_BOOK_BUILD)
	$(GITBOOK) pdf $(ROOT_DIR) $(TARGET_BOOK_BUILD)/book.pdf

# Building this book

This book is created using [GitBook] check out installation instructions on how to install it.

The kernel build system provides 3 make rules for developing the book:

```sh
make book-build   # alias for `gitbook build ./ ./target/book/www/` 
make book-serve   # alias for `gitbook serve ./ ./target/book/www/` 
make book-pdf     # alias for `gitbook pdf   ./ ./target/book/book.pdf` 
```

Note that the `pdf` rule requires having `ebook-convert` installed on your system. It is usually part of `calibre` application package.

[GitBook]: https://github.com/GitbookIO/gitbook

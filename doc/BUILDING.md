# Building guidelines

Compilation of Rust code is managed by Cargo, and building rest of files and
creating distribution packages is managed by Make. The root `Makefile` contains
all rules necessary to build source, package and run distributables and clean
build artifacts.

All build artifacts are stored in `/build` directory.

## Build configutaion

The build process is parameterized with multiple variables, whose defaults
are stored in `/misc/mk/defaults.mk`. Prior to this file, another one,
`/config.mk`, is tried to be included, and user can provide custom overrides
for these variables there. There is also possibility to override them via
environment variables. Mind to use `?=` operator when assigning variables in
overrides file, so they can be still overriden through environment.

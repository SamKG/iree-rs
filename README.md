# iree-rs

## What is this?
This crate contains rustic bindings for the [IREE](https://iree-org.github.io/iree/) runtime.

## Building
In order to build, iree-rs requires the following to be available on your machine:
- clang/clang++ (tested with v12.01, but other versions may also work)
- git

iree-rs clones and builds the [main branch of the IREE repo](https://github.com/iree-org/iree) during build time, so you don't need to have iree pre-installed on your machine

## Examples
Examples for iree-rs are available [in the repository](https://github.com/SamKG/iree-rs/tree/main/examples)

Since some examples require model weights, you may have to run [scripts](https://github.com/SamKG/iree-rs/tree/main/scripts) to get the required data ahead of time.


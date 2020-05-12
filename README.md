# fountain
### Physically based renderer

![Render of Rust logo](./images/rust_logo.png)

### Overview

A Rust path tracer based on the [PBRT 3rd edition book](https://www.pbrt.org/). Can currently
render PBRT scene files using [pbrt-parser](https://github.com/akofke/pbrt-parser) and supports
a growing subset of features.

The above Rust logo demonstrates depth of field, triangle meshes with ply file loading
(using [plydough](https://github.com/akofke/plydough)), metal material using the
Trowbridge-Reitz BSDF, and image mapped environment lighting.

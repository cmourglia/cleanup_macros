# cleanup_macros

Simple code to replace `EQ` / `NEQ` / `AND` / `OR` macros by their correct C++ equivalent (resp. `==`, `!=`, `&&`, `||`)

From the source code one can run `cargo run -- path/to/root/dir/to/parse`, otherwise just run the `cleanup_macros path/to/root/dir/to/parse`.
Replacing is done in-place on all `cpp`, `inl` and `h` files.

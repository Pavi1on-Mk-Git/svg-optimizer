### Instalacja i uruchomienie projektu

First, setup Rust and Cargo according to the [official instruction](https://doc.rust-lang.org/cargo/getting-started/installation.html).

To use commands defined in justfile, install [Just](https://github.com/casey/just#Installation).

Clone the repository: `git clone https://gitlab-stud.elka.pw.edu.pl/jprobosz/24z-zpr-svg-optimizer.git`.

From the main directory of the project, compile and run the project via `just run ARGS` or `cargo run -- ARGS`. Replace `ARGS` with command line arguments for the program.

Example: `just run examples/rect.svg` should create a `opt_rect.svg` file with just the `<svg>` tag from the original file.

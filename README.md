### Installation

First, setup Rust and Cargo according to the [official instruction](https://doc.rust-lang.org/cargo/getting-started/installation.html).

To use commands defined in justfile, install [Just](https://github.com/casey/just#Installation).

Clone the repository: `git clone https://gitlab-stud.elka.pw.edu.pl/jprobosz/24z-zpr-svg-optimizer.git`.

### Usage

From the main directory of the project, compile and run the project via `just run ARGS` or `just run-release ARGS`. Replace `ARGS` with command line arguments for the program.

Example: `just run examples/rect.svg` should create an optimized `examples/opt_rect.svg` file.

You can pass `-o` flag to set the names of the output files, like so:

`just run examples/rect.svg -o examples/rect2.svg`

By default, all optimizations except lossy ones are enabled. To disable each of them, a flag is available, for example:

`just run examples/rect.svg --no-shorten-ids`

You can also disable all optimizations by default with `-d` and only enable a select few; for example:

`just run examples/rect.svg -d --shorten-ids --remove-attribute-whitespace`

Lossy optimizations need to be explicitly enabled. The flags to achieve this are `--merge-transforms` and `--round-floats`. Precision of the floating-point numbers that they output is controlled by the `--precision` flag; by default it is set to 3.

Description of each flag is available after running `just run --help`.


# mini-nu-shell

This is just a kitbash of [mini-nu][] and [tinysh][] with some extra glue, etc.

## PROJECT STATUS

This was a funny experiment for which i sometimes find a use again.
* It is **NOT ACTIVELY MAINTAINED**.
* It is probably very buggy.
* The goal is not to be a usable interactive shell.

## Features

* Execute nu commands
  * Some are missing
  * Some, such as `use`, might be buggy
* Save variables, etc between commands
* Extremely basic text-input
  * No multi-line
  * No arrow-keys
  * Etc

## Compiling

It should usually just be `cargo build --release`.

Troubleshooting:
* Something with `uu` can't be compiled
  * `rm Cargo.lock`, and retry

## Usage

* **Run a script:** `mini-nu-shell my_file.nu`
  * It just executes the contents. `main`, fancy argument-parsing, etc wont work.
* **Run a command:** `mini-nu-shell -c 'ls | table -e'`
* **Open a interactive shell:** `mini-nu-shell`
  * **Exit:** `crtl+d`-keybind or `exit` command

## Embedding apps

You can compile a nu script into a binary "app" with this project.

### Known Problems

* Mini nu is not 100% nu compatible (see above)
* Arguments and stdin are.. something..
* It is in general a bad idea (security issues due to frozen version, etc).

### How to

1. merge your script into a single file (automation: [merge_nu_scripts][])
2. move the file to `src/script.nu`
3. `cargo build --release --features embed-app`
4. you have your binary at `target/release/mini-nu-shell`


[mini-nu]: https://github.com/cablehead/mini-nu
[tinysh]: https://github.com/zserge/tinysh
[nushell]: https://nushell.sh
[merge_nu_scripts]: https://github.com/Jan9103/merge_nu_scripts

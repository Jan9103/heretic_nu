# mini-nu-shell

This is minimal working subset of a [nu][nushell]-interpreter ([mini-nu][]) wrapped by a minimal shell-prompt (taken from [tinysh][]).  
This is just a kitbash of [mini-nu][] and [tinysh][]. All credit goes to those 2 projects (and [nushell][], etc obviously)

Please keep in mind, that this is hacked together version and many
issues you encounter are NOT nushell's fault.

## Features

* Execute nu commands
  * Some, such as `print`, are missing
  * Some, such as `use`, might work differently
* Save variables, etc between commands
* Extremely basic text-input
  * No multiline
  * No arrow-keys

So its a straight downgrade from nushell? yes, unless storage space is extremely sparse: nu0.98 is 38M and mini-nu-shell is 14M on my system.

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

### How to make things readable:

* `| to nuon`
* `| table`

why not auto-append this? because `def foo [] {}` can't be piped.


[mini-nu]: https://github.com/cablehead/mini-nu
[tinysh]: https://github.com/zserge/tinysh
[nushell]: https://nushell.sh

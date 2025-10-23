# heretic nu 2

Nu, but with zip-ties included instead of batteries included.

## PROJECT STATUS

Experiment

## Comparison to normal nu

* Completely different REPL input
  * You control literally everything about key-input (just overwrite the `_heretic_nu_input` function)
  * The prompt can do anything and does not get overdrawn
  * The example input does not contain a tab-completion
* Added `-x` mode (launch it with `-x` or `-xx` to see what is happening under the hood)
* `evil` command (evaluate strings as code)
* Probably lots of bugs and missing things (no plugins, etc)

## Credits

This is a fork of [mini-nu-shell](https://github.com/jan9103/mini-nu-shell),
which intern is based on [mini-nu](https://github.com/cablehead/mini-nu).

This project is built around [nushell][].


[nushell]: https://nushell.sh

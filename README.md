# heretic nu 2

Nu, but with zip-ties instead of batteries included.

## PROJECT STATUS

* Experiment
* Usable (as debugger for nu)
* Not actively maintained

## Comparison to normal nu

* Completely different REPL input
  * You control literally everything about key-input (just overwrite the `_heretic_nu_input` function)
  * The prompt can do anything and does not get overdrawn
  * The example input does not contain a tab-completion
* Debugging stuff:
  * debug mode: `x` (get a rough idea where in the code it is)
  * debug mode: `xx` (see which IR step it is currently running)
  * debug mode: `step` (WIP, inspect the engine state during individual IR steps in a separate window)
  * debug mode: `off`
  * launch-arguments: `-x`, `-xx`
  * command: `heretic debug` (switch modes mid-execution)
* different config system:
  1. config file: `~/.config/heretic_nu/config.nu`
  1. each `.nu` file in a directory specified by `$env.heretic_nu_autoload_dirs` (yes you can edit it in your main `config.nu`)
* `evil` command (evaluate strings as code)
  * `heretic const evil`: evaluate strings as code at const-time..
* builtin extended [commtest](https://github.com/jan9103/commtest):
  * `#[test]` to mark a function as a test
  * `#[test_param] flag-name = ['list' 'of' 'values' 'in' 'nuon' 'format']` (concept "inspired" by [pytest](https://docs.pytest.org/en/7.1.x/example/parametrize.htmlhttps://docs.pytest.org/en/7.1.x/example/parametrize.html))
  * `heretic tests run` to run all tests in scope
* Probably lots of bugs and missing things (no plugins, etc)

## Credits

This is a fork of [mini-nu-shell](https://github.com/jan9103/mini-nu-shell),
which intern is based on [mini-nu](https://github.com/cablehead/mini-nu).

This project is built around [nushell][].


[nushell]: https://nushell.sh

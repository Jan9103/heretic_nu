def --env _heretic_nu_prompt []: nothing -> string {
  for pph in ($env.config?.hooks?.pre_prompt? | default []) {
    do --env $pph | ignore
  }

  if "PROMPT_COMMAND" in $env {
    do $env.PROMPT_COMMAND
    | $"($in)($env.PROMPT_INDICATOR? | default '')"
  } else {
    $"\n(pwd)\n> "
  }
}

def --env _heretic_nu_input []: nothing -> string {
  let res = do --env {
    print --no-newline "\e[s\e[0J"
    def render [text: string, cursor: int]: nothing -> nothing {
      let cmov: int = (($text | str length) - $cursor) + 1
      print --no-newline $"\e[u\e[0J($'($text) ' | nu-highlight)\e[($cmov)D"
    }

    mut history_nidx: int = 0
    mut history: list<string> = ($env._HERETIC_NU_HISTORY? | default [] | prepend '')
    mut text: string = ""
    mut cursor: int = 0

    loop {
      render $text $cursor

      let input = (input listen --types ['key' 'paste'])
      if $input.type == 'key' {
        if $input.key_type == 'char' and ($input.modifiers | where $it != 'keymodifiers(shift)') == [] {
          $text = $'(if $cursor != 0 {$text | str substring ..($cursor - 1)})($input.code)($text | str substring $cursor..)'
          $cursor = ($cursor + 1)
          continue
        }
        if $input.key_type == 'other' {
          if $input.code == 'backspace' {
            if $cursor > 0 {
              if $input.modifiers == ['keymodifiers(alt)'] {
                let e = ($text | str substring ..($cursor - 1) | split row ' ' | drop 1 | str join ' ' | str length)
                $text = $"(if $e == 0 {''} else { $text | str substring ..($e - 1) })($text | str substring ($cursor)..)"
                $cursor = $e
              } else {
                $text = $'(if $cursor != 1 {$text | str substring ..($cursor - 2)})($text | str substring $cursor..)'
                $cursor = ($cursor - 1)
              }
            }
            continue
          }
          if $input.code == 'delete' {
            $text = $'(if $cursor != 0 {$text | str substring ..($cursor - 1)})($text | str substring ($cursor + 1)..)'
            continue
          }
          if $input.code == 'left' {
            if $input.modifiers == ['keymodifiers(alt)'] {
              $cursor = ($text | str substring ..($cursor - 1) | split row ' ' | drop 1 | str join ' ' | str length)
            } else {
              $cursor = ([($cursor - 1) 0] | math max)
            }
            continue
          }
          if $input.code == 'right' {
            if $input.modifiers == ['keymodifiers(alt)'] {
              $cursor = (
                ($text | str length) - ($text | str substring ($cursor + 1).. | split row ' ' | skip 1 | str join ' ' | str length)
                | if $in != ($text | str length) { $in - 1 } else { $in }
              )
              # $cursor = $tl - ($text | str reverse | str substring ..(($tl - $cursor) - 1) | split row ' ' | drop 1 | str join ' ' | str length)
            } else {
              $cursor = ([($cursor + 1) ($text | str length)] | math min)
            }
            continue
          }
          if $input.code == 'enter' {
            print ''
            $env._HERETIC_NU_HISTORY = ($env._HERETIC_NU_HISTORY? | default [] | prepend $text)
            return $text
          }
          if $input.code == 'up' {
            if (($history | length) - 1) <= $history_nidx {
              continue
            }
            $history = ($history | update $history_nidx $text)
            $history_nidx = ($history_nidx + 1)
            $text = ($history | get $history_nidx)
            $cursor = ($text | str length)
            continue
          }
          if $input.code == 'down' {
            if $history_nidx == 0 { continue }
            $history = ($history | update $history_nidx $text)
            $history_nidx = ($history_nidx - 1)
            $text = ($history | get $history_nidx)
            $cursor = ([($text | str length) ($cursor)] | math min)
            continue
          }
          if $input.code in ['tab' 'esc'] {
            # common typos
            continue
          }
        }
        if $input.modifiers? == ['keymodifiers(control)'] {
          if $input.code? == 'd' {
            print ''
            exit
          }
          if $input.code? == 'c' {
            return ''
          }
        }

        # TODO
      }
      if $input.type == 'paste' {
        let ic = ($input.content | str replace --all "\n" ' ' | ansi strip)
        $text = $'(if $cursor != 0 {$text | str substring ..($cursor - 1)})($ic)($text | str substring $cursor..)'
        $cursor = ($cursor + ($ic | str length))
        continue
      }
      error make {
        msg: $'Input not handled: ($input | to nuon --raw)'
      }
    }
  }

  for peh in ($env.config?.hooks?.pre_execution? | default []) {
    do --env $peh | ignore
  }

  $res
}

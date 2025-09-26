def _mini_nu_prompt []: nothing -> string {
  "\n\n> "
}

def --env _mini_nu_input []: nothing -> string {
  print --no-newline "\e[s\e[0J"
  def render [text: string, cursor: int]: nothing -> nothing {
    let cmov: int = (($text | str length) - $cursor) + 1
    print --no-newline $"\e[u\e[0J($'($text) ' | nu-highlight)\e[($cmov)D"
  }

  mut history_nidx: int = 0
  mut history: list<string> = ($env._MINI_NU_HISTORY? | default [] | prepend '')
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
            $text = $'(if $cursor != 1 {$text | str substring ..($cursor - 2)})($text | str substring $cursor..)'
            $cursor = ($cursor - 1)
          }
          continue
        }
        if $input.code == 'delete' {
          $text = $'(if $cursor != 0 {$text | str substring ..($cursor - 1)})($text | str substring ($cursor + 1)..)'
          continue
        }
        if $input.code == 'left' {
          $cursor = ([($cursor - 1) 0] | math max)
          continue
        }
        if $input.code == 'right' {
          $cursor = ([($cursor + 1) ($text | str length)] | math min)
          continue
        }
        if $input.code == 'enter' {
          print ''
          $env._MINI_NU_HISTORY = ($env._MINI_NU_HISTORY? | default [] | prepend $text)
          return $text
        }
        if $input.code == 'up' {
          if (($history | length) - 1) <= $history_nidx {
            print ">=\n\n\n\n"
            continue
          }
          $history = ($history | update $history_nidx $text)
          $history_nidx = ($history_nidx + 1)
          $text = ($history | get $history_nidx)
          $cursor = ([($text | str length) ($cursor)] | math min)
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
      if $input.code? == 'd' and $input.modifiers == ['keymodifiers(control)'] {
        print ''
        exit
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
  "" # make LSP happy
}

def main_loop [socket_dir: path]: nothing -> nothing {
  let no_data_lock: path = ($socket_dir | path join 'no_data_lock.bin')
  let no_return_lock: path = ($socket_dir | path join 'no_return_lock.bin')
  # let response_file: path = ($socket_dir | path join 'response.txt')
  # let data_file: path = ($socket_dir | path join 'data.nuon')
  let render_file: path = ($socket_dir | path join 'render.txt')

  loop {
    clear
    print "Awaiting data.."
    loop {
      try {
        "" | save $no_data_lock
        rm $no_data_lock
        break
      }
      sleep 0.2sec
    }

    clear
    print (open $render_file)
    rm $render_file

    while (input listen --types ['key']).code? != 'enter' {}
    # run_ui $data_file | save --raw $response_file
    touch $no_data_lock
    rm $no_return_lock
  }
}

main_loop $env.socket_dir

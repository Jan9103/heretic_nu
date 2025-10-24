let In = $in
let sock_dir = $env.sock_dir

let lock = ($sock_dir | path join "no_return_lock.bin")
$In | save ($sock_dir | path join "render.txt")
touch $lock
rm ($sock_dir | path join 'no_data_lock.bin')

loop {
  try {
    "" | save --raw $lock
    rm $lock
    break
  }
  sleep 0.2sec
}

# let response = ($sock_dir | path join 'response.txt' | str trim)
# rm ($sock_dir | path join 'response.txt')

# $response

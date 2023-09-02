# tidybee-backend-rust
Watch for changes in directories and recursively list directories.

## Tools
- rust: the language
- cargo: the language package manager
- nc: netcat for testing purposes only

## Before you run
Launch a server listening on port 8080:
```
# this is the way I did it on macOS
while true; do nc -l 8080; done
```

## Build & Run
As usual, you have two ways to build and run a program:
```
# build
cargo build
# run
./target/debug/tidybee <commands>
```
Shorter:
```
# build & run
cargo run <commands>
```

## Mandatory commands
```
-l, --list <dir>    # recursively list the given directories, send it to localhost:8080 (json) and exit
-w, --watch <dir>   # watch for changes in the given directories and send it to localhost:8080 (json)
```

## Optional commands
Optional commands are parsed in the configuration module, but not implemented!
```
-e, --extension <exts>  # file extensions to list/watch (default: all)
-t, --type <types>      # file types to list/watch (default: all)
-r, --receive <addr>    # receive json data from this address (file manager stuff)
-s, --send <addr>       # send json data to this address (send file list/events)
```

## Run the test suite
```
cargo test
```

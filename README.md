# tidybee-backend
[TidyBee](https://github.com/tidybee)'s backend repository.

## Usage
```
Usage: tidybee <ARGUMENTS>

Arguments:

  REQUIRED ARGUMENTS
    -l, --list <dir>          Recursively list directory
    -w, --watch <dir>         Watch for changes in directory

  OPTIONAL ARGUMENTS
    -r, --receive <port>      Receive file system actions in JSON format (default: 8079)
    -s, --send <addr>         Send listing or events in JSON format (default: localhost:8080)
    -t, --type <type>         List or watch only these file types: all, directory, regular (it defaults to all)

Example:

    tidybee -w /Users/john/Desktop,/Users/john/Documents -e pdf,docx
    tidybee -l "$HOME" -t regular -s tbfront:80
```

## Build
```
cargo build
```

## Specifications
Here to keep a track of the different technologies the backend is using.

### Crates
| Crate | Version | Description |
| - | - | - |
| [Notify](https://docs.rs/notify/latest/notify/) | 6.0.1 | Watch for file events inside a given directory |
| [serde_json](https://docs.rs/serde_json/latest/serde_json/) | v |  Serialize Rust data structure, deserialize typed json |
| [Tokio](https://docs.rs/tokio/latest/tokio/) | v |  d |

### Tested hosts (build and execution)
| Platform | Processor | Rust version |
| - | - | - |
| macOS | arm | 1.71.1 |

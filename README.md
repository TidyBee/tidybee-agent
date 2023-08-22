# tidybee-backend
[TidyBee](https://github.com/tidybee)'s backend repository. Watch for changes in directories and recursively list directories.

## Usage
```
USAGE:
    tidybee [OPTIONS] --list <DIRECTORIES>... --watch <DIRECTORIES>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --extension <EXTENSIONS>...    Specify file extensions
    -l, --list <DIRECTORIES>...        Specify directories for listing
    -r, --receive <ADDRESS>            Specify receive address
    -s, --send <ADDRESS>               Specify send address
    -t, --type <TYPES>...              Specify file types
    -w, --watch <DIRECTORIES>...       Specify directories for watching

EXAMPLE:
    tidybee -w /Users/john/Desktop,/Users/john/Documents -e pdf,docx
    tidybee -l "$HOME" -t regular -s tbfront:80
```

## Build
```
cargo build
```

## Run the test suite
```
cargo test
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

## Known issues
### file listing
- does not handle bad permissions
### file event watcher
- does not handle bad permissions

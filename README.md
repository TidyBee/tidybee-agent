# tidybee-backend
[TidyBee](https://github.com/tidybee)'s backend repository.

## Features
### FS Events Watcher
1. Watch for file events inside a given directory.
2. Decide whether or not it is a pertinent information.
3. Send the data to the front client.

### Directory Listing
1. .
2. .
3. .

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

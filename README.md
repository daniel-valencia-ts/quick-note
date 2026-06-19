# QuickNote TUI

A simple, compact terminal-based note-taking tool.

## Installation

### crates.io

Install it via Rust's package manager by running:
```sh
cargo install quick-note
```

### From source

Install `quick-note`'s source code and compile it yourself.

1. Clone this repository to your local machine by using the command: 
```bash
git clone https://github.com/daniel-valencia-ts/quick-note.git
``` 

2. From the cloned repository, run:
```bash
cargo run
```

## Usage

This app is structured in several modes that let you access its features. 
After having run it, you'll be in the `display` mode, whose purpose is to show a note on screen. 
Furthermore, from here you can execute several commands based on the action you want to perform.

![Example of the display mode](https://i.postimg.cc/CLzmm6HB/display-mode.png)

To create a new note, use the command `add`, which takes you to a menu in which you'll be asked
for the title of the note you're creating, its body, and how relevant it is.

![Example of the process of adding a new note](https://i.postimg.cc/0NgW6dSM/add-mode.png)

As you add more notes, the `select` mode will be helpful to pick the one that you want to be displayed.

![Example of the select mode](https://i.postimg.cc/Kv6rQcYh/select-mode.png)

If a note isn't helpful anymore, it can be easily removed.

![Example of the remove mode](https://i.postimg.cc/wMP5CgcP/remove-mode.png)

Furthermore, the content of an existing note can be modified.

![Example of the modify mode](https://i.postimg.cc/sXT4JpMb/modify-mode.png)

## License

This project is licensed under the MIT License.
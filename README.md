# Atlas Text Editor

Atlas is a **fast**, **lightweight** text editor inspired by the philosophy of Emacs, Neovim, and Zed. Its main goal is to be **simple to set up**, **simple to use**, and **optionally modal**—allowing you to switch between editing and command modes if you desire, while still being able to operate in a more traditional "insert mode only" style.

## Getting Started
**NOTE**: You must have [Rust](https://www.rust-lang.org/tools/install) installed.

1. **Clone** the repository:
   ```bash
   git clone https://github.com/misterclayt0n/atlas
   ```
2. **Build** the project:
   ```bash
   cd atlas
   cargo build --release
   ```
3. **Run** the binary:
   ```bash
   cargo run --release
   ```

## Development
For development, you can use:
```bash
cargo run
```

## Roadmap / TODO

Not ordered by priority.

- [ ] **Basic text editing**: Implement fundamental text editing operations
  - [x] Basic text display
  - [ ] Text input handling
  - [ ] Cursor movement
  - [ ] Basic editing operations (insert, delete)
- [ ] **Modal editing**:
  - [ ] Basic mode switching (Normal/Insert)
  - [ ] Command mode
  - [ ] Visual mode
- [ ] **Buffer Management**:
  - [ ] Multiple buffers support
  - [ ] Buffer switching
  - [ ] File loading/saving
- [ ] **UI Improvements**:
  - [ ] Status line
  - [ ] Line numbers
  - [ ] Cursor visualization
- [ ] **Advanced Features**:
  - [ ] Syntax highlighting
  - [ ] Search and replace
  - [ ] Undo/Redo
  - [ ] Split views
- [ ] **Configuration**:
  - [ ] Customizable keybindings
  - [ ] Themes
  - [ ] User configuration file

## Contributing

Feel free to open an [issue](https://github.com/misterclayt0n/atlas/issues) or submit a pull request if you have ideas or bug fixes.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

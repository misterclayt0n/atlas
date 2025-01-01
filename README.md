# Atlas Text Editor

Atlas is a **fast**, **lightweight** text editor inspired by the philosophy of Emacs, Neovim, and Zed. Its main goal is to be **simple to set up**, **simple to use**, and **optionally modal**—allowing you to switch between editing and command modes if you desire, while still being able to operate in a more traditional “insert mode only” style.

## Getting Started
**NOTE**: You must have the [Odin](https://odin-lang.org/) compiler installed.

1. **Clone** the repository:
   ```bash
   git clone https://github.com/misterclayt0n/atlas
   ```
2. **Build** the project (Release version):
   ```bash
   cd atlas
   ./build_release.sh
   ```
3. **Run** the binary:
   ```bash
   ./atlas
   ```

## Roadmap / TODO

Not ordered.

- [ ] **Complete text viewer**
- [x] **Hot reload**: Hot reload setup for easier configuration/development.
- [ ] **Modal editing**
- [ ] **Configurable keybindings**: Allow users to customize shortcuts (Something inspired by the [Focus](https://github.com/focus-editor/focus) editor way of configuring).
- [ ] **Advanced movement**: Add more advanced navigation (e.g., jump to beginning/end of line, next word, etc.).
- [ ] **Text selection** (visual mode) and basic cut/copy/paste.
- [ ] **Multiple buffers/tabs**: Handle more than one file in the same session.
- [ ] **Syntax highlighting**: Provide at least a minimal form or plugin system for highlighting.

## Contributing

Feel free to open an [issue](https://github.com/misterclayt0n/atlas/issues) or submit a pull request if you have ideas or bug fixes.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

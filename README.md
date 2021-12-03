# NES-emulator

![CI](https://github.com/Laastine/nes-emulator/workflows/nes-emulator/badge.svg?branch=master&event=push)

<p float="left">
<img src="assets/zelda.png" width="30%">
<img src="assets/smb3.png" width="30%">
<img src="assets/metroid.png" width="30%">
</p>

Learning project to build Nintendo Entertainment System emulator. The goal is to build NES emulator using only rust libraries so no SDL-library requirement.

## Controls

`Arrow keys` - D-pad<br>
`S` - Start<br>
`A` - Select<br>
`Z` - Button B<br>
`X` - Button A<br>
`R` - Reset<br>
`Esc` - Quit  

## TODO

- [x] CPU
- [x] Display RAM & CPU status in terminal for debugging
- [x] ROM reader & mapper
- [x] Graphics window
- [x] Controls<br>
- [x] PPU (Pixel Processing Unit)<br>
  - [x] PPU background rendering
  - [x] PPU sprites
- [x] More ROM mappers
- [x] Cross-platform (<strike>MacOS</strike>, Linux, Windows)

 ✍️ APU (Audio Processing Unit)<br/>

## Usage

```
USAGE:
nes-emulator [FLAGS]

FLAGS:
-h, --help                      Prints help information
-v, --version                   Prints version information
-r, --rom                       Rom filename to load
-d, --debug                     Show memory debug on terminal
```

### Quick testing

`cargo run --release -- --rom rom-file-here`

## References

- [Nesdev Wiki](http://wiki.nesdev.com/w/index.php/Nesdev_Wiki)<br>
- [javidx9 NES-tutorial](https://www.youtube.com/watch?v=nViZg02IMQo&list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf)<br>
- [6502 Assembler](https://www.masswerk.at/6502/assembler.html)
- Lot's of other emulators

# rust-chip8-opengl
A CHIP-8 emulator written in rust.
Can be ran either in terminal or using GLFW to render the screen to a window.

## Usage
`cargo run -- [options]`
Available options:
* `-f, --file [FILE]`: The CHIP-8 file to run, i.e. `-f ./my_game.ch8`.
  Can be omitted to run the emulator in an interactive mode where the use enters opcodes manually.
* `-m, --mode [MODE]`: The mode to run the emulator in, either `terminal` or `open-gl`.
* `--debug-file [FILE]`: The file to log the opcodes to. If omitted, no opcodes are logged.

## Examples
`cargo run -- -f ./Chip8Picture.ch8 -m terminal`
```
[][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][]
[][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                  [][][][][][][][]    []            []    []    [][][][][][][][]    [][][][][][][][]                    [][]
[][]                  []                  []            []    []    []            []    []            []                    [][]
[][]                  []                  []            []    []    []            []    []            []                    [][]
[][]                  []                  []            []    []    []            []    []            []                    [][]
[][]                  []                  []            []    []    []            []    []            []                    [][]
[][]                  []                  []            []    []    []            []    []            []                    [][]
[][]                  []                  []            []    []    []            []    []            []                    [][]
[][]                  []                  [][][][][][][][]    []    [][][][][][][][]    [][][][][][][][]                    [][]
[][]                  []                  []            []    []    []                  []            []                    [][]
[][]                  []                  []            []    []    []                  []            []                    [][]
[][]                  []                  []            []    []    []                  []            []                    [][]
[][]                  []                  []            []    []    []                  []            []                    [][]
[][]                  []                  []            []    []    []                  []            []                    [][]
[][]                  []                  []            []    []    []                  []            []                    [][]
[][]                  [][][][][][][][]    []            []    []    []                  [][][][][][][][]                    [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][]                                                                                                                        [][]
[][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][]
[][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][][]

  PC  |  I   |  V0  |  V1  |  V2  |  V3  |  V4  |  V5  |  V6  |  V7  |  V8  |  V9  |  Va  |  Vb  |  Vc  |  Vd  |  Ve  |  Vf  |
 0x246| 0x294| 0x2C |  0x8 | 0x10 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |  0x0 |
  DT  |  ST  |  I0  |  I1  |  I2  |  I3  |  I4  |  I5  |  I6  |  I7  |  I8  |  I9  |  IA  |  IB  |  IC  |  ID  |  IE  |  IF  
  0x0 |  0x0 |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   |  F   
```

`cargo run -- -f Chip8Picture.ch8 -m open-gl`

![Screenshot of a CHIP-8 ROM being rendered using open-gl](opengl-screenshot.png)
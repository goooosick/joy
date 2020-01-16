# Joy
Just another Gameboy emulator, in Rust. The cpu is cycle accurate, passed blargg's cpu_instrs and instr_timing tests, but not others. Apu and ppu have been implemented. 

## Run
```sh
	cargo run --release rom_file -s scale
```

## Screenshot
[Bad Apple](http://forums.nesdev.com/viewtopic.php?f=20&t=18688)

![bad_apple](res/bad_apple.png)

References:

[op codes](https://pastraiser.com/cpu/gameboy/gameboy_opcodes.html)

[gbdev](https://gbdev.gg8.se/wiki/articles/Main_Page)

[Pan Docs](http://problemkaputt.de/pandocs.htm)

[codeslinger's toturial](http://www.codeslinger.co.uk/pages/projects/gameboy/beginning.html)

[imrannazar's emulation in js](http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-The-CPU)

others in comments

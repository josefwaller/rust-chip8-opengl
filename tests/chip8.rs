#[cfg(test)]
mod tests {
    use rust_chip8_opengl::chip8::Chip8;

    #[test]
    fn test_ld_v_byte() {
        let mut emu = Chip8::new();
        const INST: u16 = 0x6022;
        emu.step(INST);
        assert_eq!(emu.get_register_value(0), 0x22);
    }
}

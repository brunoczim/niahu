use super::*;

#[test]
fn sub_algo() {
    let mut vm = Machine::new();
    vm.mem[0x0] = LDR;
    vm.mem[0x1] = 0x81;
    vm.mem[0x2] = NOT;
    vm.mem[0x3] = ADD;
    vm.mem[0x4] = 0x83;
    vm.mem[0x5] = ADD;
    vm.mem[0x6] = 0x80;
    vm.mem[0x7] = STR;
    vm.mem[0x8] = 0x82;
    vm.mem[0x9] = HLT;

    vm.mem[0x80] = 150;
    vm.mem[0x81] = 3;
    vm.mem[0x83] = 1;

    vm.execute();

    assert_eq!(vm.mem[0x82], 147);
    assert_eq!(vm.cycles, 6);
    assert_eq!(vm.accesses, 14);
}

#[test]
fn mul_algo() {
    let mut vm = Machine::new();
    vm.mem[0x0] = LDR;
    vm.mem[0x1] = 0x85;
    vm.mem[0x2] = STR;
    vm.mem[0x3] = 0x82;
    vm.mem[0x4] = LDR;
    vm.mem[0x5] = 0x81;
    vm.mem[0x6] = STR;
    vm.mem[0x7] = 0x83;
    vm.mem[0x8] = JZ;
    vm.mem[0x9] = 0x18;
    vm.mem[0xA] = ADD;
    vm.mem[0xB] = 0x84;
    vm.mem[0xC] = STR;
    vm.mem[0xD] = 0x83;
    vm.mem[0xE] = LDR;
    vm.mem[0xF] = 0x80;
    vm.mem[0x10] = ADD;
    vm.mem[0x11] = 0x82;
    vm.mem[0x12] = STR;
    vm.mem[0x13] = 0x82;
    vm.mem[0x14] = LDR;
    vm.mem[0x15] = 0x83;
    vm.mem[0x16] = JMP;
    vm.mem[0x17] = 0x8;
    vm.mem[0x18] = HLT;

    vm.mem[0x80] = 5;
    vm.mem[0x81] = 11;
    vm.mem[0x84] = 255;
    vm.mem[0x85] = 0;

    vm.execute();

    assert_eq!(vm.mem[0x82], 55);
    assert_eq!(vm.cycles, 94);
    assert_eq!(vm.accesses, 257);
}

#[test]
fn is_pos() {
    let mut vm = Machine::new();
    vm.mem[0x0] = LDR;
    vm.mem[0x1] = 0x80;
    vm.mem[0x2] = NOT;
    vm.mem[0x3] = JN;
    vm.mem[0x4] = 0xA;
    vm.mem[0x5] = LDR;
    vm.mem[0x6] = 0x83;
    vm.mem[0x7] = STR;
    vm.mem[0x8] = 0x81;
    vm.mem[0x9] = HLT;
    vm.mem[0xA] = LDR;
    vm.mem[0xB] = 0x82;
    vm.mem[0xC] = STR;
    vm.mem[0xD] = 0x81;
    vm.mem[0xE] = HLT;

    vm.mem[0x80] = 128;
    vm.mem[0x82] = 1;
    vm.mem[0x83] = 0;

    vm.execute();

    assert_eq!(vm.mem[0x81], 0);
}

#[test]
fn save_load_mem() {
    let mut vm = Machine::new();
    vm.mem[0x0] = 42;
    vm.mem[0xB5] = 220;
    let mut buf = Vec::new();
    vm.save_mem(&mut buf).unwrap();
    let mut vm2 = Machine::new();
    vm2.load_mem(&mut &buf[..]).unwrap();
    assert_eq!(&vm.mem as &[_], &vm2.mem as &[_]);
}

#[test]
fn save_load_state() {
    let mut vm = Machine::new();
    vm.mem[0x0] = 42;
    vm.mem[0xB5] = 220;
    vm.pc = 0x5;
    vm.ra = 203;
    vm.ri = 0x12;
    vm.cycles = 2;
    vm.accesses = 6;
    let mut buf = Vec::new();
    vm.save_state(&mut buf).unwrap();
    let mut vm2 = Machine::new();
    vm2.load_state(&mut &buf[..]).unwrap();
    assert_eq!(vm, vm2);
}

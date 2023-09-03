#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum Symbols {
    EMPTY = 0x00,
    SIM_0 = 0xFC, // 0
    SIM_1 = 0x60, // 1
    SIM_2 = 0xDA, // 2
    SIM_3 = 0xF2, // 3
    SIM_4 = 0x66, // 4
    SIM_5 = 0xB6, // 5
    SIM_6 = 0xBE, // 6
    SIM_7 = 0xE0, // 7
    SIM_8 = 0xFE, // 8
    SIM_9 = 0xF6, // 9
    SIM_A = 0xEE, // A
    SIM_b = 0x3E, // b
    SIM_C = 0x9C, // C
    SIM_d = 0x7A, // d
    SIM_E = 0x9E, // E
    SIM_F = 0x8E, // F
    SIM_G = 0xBC, // G
    SIM_H = 0x2E, // H
    SIM_I = 12, // I
    SIM_J = 0x78, // J
    SIM_K = 0xAE, // K
    SIM_L = 0x1C, // L
    SIM_M = 0xA8, // M
    SIM_N = 0xEC, // N
    SIM_P = 0xCE, // P
    SIM_Q = 0xE6, // Q
    SIM_R = 204, // R
    SIM_T = 0x1E, // T
    SIM_U = 0x7C, // U
    SIM_V = 0x38, // V
    SIM_W = 0x54, // W
    SIM_X = 0x6E, // X
    SIM_Y = 0x76, // Y
    LINE = 0x02, // -
    BOTTOM_LINE = 0x10, // _
    SIM_B = 0xFF, // B.
    SIM_D = 0xFD, // D.
    POINT = 0x01
}
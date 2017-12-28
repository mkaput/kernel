pub const NULL_KEY: u8 = 0;
pub const Q_PRESSED: u8 = 0x10;
pub const Q_RELEASED: u8 = 0x90;
pub const W_PRESSED: u8 = 0x11;
pub const W_RELEASED: u8 = 0x91;
pub const E_PRESSED: u8 = 0x12;
pub const E_RELEASED: u8 = 0x92;
pub const R_PRESSED: u8 = 0x13;
pub const R_RELEASED: u8 = 0x93;
pub const T_PRESSED: u8 = 0x14;
pub const T_RELEASED: u8 = 0x94;
pub const Z_PRESSED: u8 = 0x15;
pub const Z_RELEASED: u8 = 0x95;
pub const U_PRESSED: u8 = 0x16;
pub const U_RELEASED: u8 = 0x96;
pub const I_PRESSED: u8 = 0x17;
pub const I_RELEASED: u8 = 0x97;
pub const O_PRESSED: u8 = 0x18;
pub const O_RELEASED: u8 = 0x98;
pub const P_PRESSED: u8 = 0x19;
pub const P_RELEASED: u8 = 0x99;
pub const A_PRESSED: u8 = 0x1E;
pub const A_RELEASED: u8 = 0x9E;
pub const S_PRESSED: u8 = 0x1F;
pub const S_RELEASED: u8 = 0x9F;
pub const D_PRESSED: u8 = 0x20;
pub const D_RELEASED: u8 = 0xA0;
pub const F_PRESSED: u8 = 0x21;
pub const F_RELEASED: u8 = 0xA1;
pub const G_PRESSED: u8 = 0x22;
pub const G_RELEASED: u8 = 0xA2;
pub const H_PRESSED: u8 = 0x23;
pub const H_RELEASED: u8 = 0xA3;
pub const J_PRESSED: u8 = 0x24;
pub const J_RELEASED: u8 = 0xA4;
pub const K_PRESSED: u8 = 0x25;
pub const K_RELEASED: u8 = 0xA5;
pub const L_PRESSED: u8 = 0x26;
pub const L_RELEASED: u8 = 0xA6;
pub const Y_PRESSED: u8 = 0x2C;
pub const Y_RELEASED: u8 = 0xAC;
pub const X_PRESSED: u8 = 0x2D;
pub const X_RELEASED: u8 = 0xAD;
pub const C_PRESSED: u8 = 0x2E;
pub const C_RELEASED: u8 = 0xAE;
pub const V_PRESSED: u8 = 0x2F;
pub const V_RELEASED: u8 = 0xAF;
pub const B_PRESSED: u8 = 0x30;
pub const B_RELEASED: u8 = 0xB0;
pub const N_PRESSED: u8 = 0x31;
pub const N_RELEASED: u8 = 0xB1;
pub const M_PRESSED: u8 = 0x32;
pub const M_RELEASED: u8 = 0xB2;
pub const ZERO_PRESSED: u8 = 0x29;
pub const ONE_PRESSED: u8 = 0x2;
pub const NINE_PRESSED: u8 = 0xA;
pub const POINT_PRESSED: u8 = 0x34;
pub const POINT_RELEASED: u8 = 0xB4;
pub const SLASH_RELEASED: u8 = 0xB5;
pub const BACKSPACE_PRESSED: u8 = 0xE;
pub const BACKSPACE_RELEASED: u8 = 0x8E;
pub const SPACE_PRESSED: u8 = 0x39;
pub const SPACE_RELEASED: u8 = 0xB9;
pub const ENTER_PRESSED: u8 = 0x1C;
pub const ENTER_RELEASED: u8 = 0x9C;

static _qwertyuiop: &[u8] = b"qwertyuiop";
// 0x10-0x1c
static _asdfghjkl: &[u8] = b"asdfghjkl";
static _zxcvbnm: &[u8] = b"zxcvbnm";
static _num: &[u8] = b"123456789";

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyCode(pub u8);

impl From<KeyCode> for u8 {
    fn from(key_code: KeyCode) -> u8 {
        match key_code.0 {
            0x1c => b'\n',
            0x38 => b' ',
            0xe => b'\r',
            POINT_RELEASED => b'.',
            SLASH_RELEASED => b'/',
            ZERO_PRESSED => b'0',
            key @ ONE_PRESSED...NINE_PRESSED => _num[key as usize - ONE_PRESSED as usize],
            key @ 0x10...0x1C => _qwertyuiop[key as usize - 0x10],
            key @ 0x1e...0x26 => _asdfghjkl[key as usize - 0x1e],
            key @ 0x2c...0x32 => _zxcvbnm[key as usize - 0x2c],
            _ => 0,
        }
    }
}

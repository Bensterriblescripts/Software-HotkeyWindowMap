use winapi::um::winuser::*;

fn get_key_code(key: &str) -> i32 {
    match key {
        "A" => 0x1E,
        "B" => 0x30,
        "C" => 0x2E,
        "CTRL"  => VK_CONTROL,
        "ALT" => VK_MENU,
        "SHIFT" => VK_SHIFT,
        "WIN" => VK_LWIN,
        _ => 0x00,
    }
}
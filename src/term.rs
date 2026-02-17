/// Detect terminal size via ioctl. Returns (columns, rows).
pub fn size() -> Option<(usize, usize)> {
    #[cfg(unix)]
    {
        #[repr(C)]
        struct Winsize {
            ws_row: u16,
            ws_col: u16,
            ws_xpixel: u16,
            ws_ypixel: u16,
        }

        #[cfg(target_os = "macos")]
        const TIOCGWINSZ: u64 = 0x40087468;
        #[cfg(target_os = "linux")]
        const TIOCGWINSZ: u64 = 0x5413;

        unsafe extern "C" {
            fn ioctl(fd: i32, request: u64, ...) -> i32;
        }

        unsafe {
            let mut ws = std::mem::zeroed::<Winsize>();
            let ret = ioctl(2, TIOCGWINSZ, &mut ws as *mut Winsize);
            if ret == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
                Some((ws.ws_col as usize, ws.ws_row as usize))
            } else {
                None
            }
        }
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_returns_something_or_none() {
        let _ = size();
    }
}

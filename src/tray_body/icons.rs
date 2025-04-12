use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use std::{ffi::c_void, mem, ptr};
use anyhow::{bail, Result};

pub fn icon_to_hbitmap(hicon: HICON) -> Result<HBITMAP> {
    unsafe {
        let hdc = GetDC(0);

        let mut bmi: BITMAPINFO = mem::zeroed();
        bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = 16;
        bmi.bmiHeader.biHeight = -16; // ujemna wysokość = top-down bitmapa
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32; // 32-bitowa bitmapa = RGBA
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut bits_ptr: *mut c_void = ptr::null_mut();

        let hbitmap = CreateDIBSection(
            hdc,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            0,
            0,
        );

        let mem_dc = CreateCompatibleDC(hdc);
        let old = SelectObject(mem_dc, hbitmap as _);

        let ok = DrawIconEx(
            mem_dc,
            0,
            0,
            hicon,
            16,
            16,
            0,
            0,
            DI_NORMAL,
        );

        SelectObject(mem_dc, old);
        DeleteDC(mem_dc);
        ReleaseDC(0, hdc);

        if ok == 0 || hbitmap == 0 {
            bail!("DrawIconEx failed or bitmap is null");
        }

        Ok(hbitmap)
    }
}
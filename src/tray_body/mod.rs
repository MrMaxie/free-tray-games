// Inspired by the tray-item crate: https://github.com/olback/tray-item-rs

mod icons;
mod funcs;
mod structs;

use std::{
    cell::RefCell,
    mem,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread,
};
use anyhow::{bail, Result, Context};
use windows_sys::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::{
        Shell::{Shell_NotifyIconW, NIF_ICON, NIF_TIP, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW},
        WindowsAndMessaging::{
            InsertMenuItemW,
            LoadImageW,
            PostMessageW,
            HICON,
            IMAGE_ICON,
            LR_DEFAULTCOLOR,
            MENUITEMINFOW,
            MFS_DISABLED,
            MFS_UNHILITE,
            MFT_SEPARATOR,
            MFT_STRING,
            MIIM_BITMAP,
            MIIM_FTYPE,
            MIIM_ID,
            MIIM_STATE,
            MIIM_STRING,
            WM_DESTROY
        },
    },
};

use icons::*;
use funcs::*;
use structs::*;

thread_local!(static WININFO_STASH: RefCell<Option<WindowsLoopData>> = RefCell::new(None));

type CallBackEntry = Option<Box<dyn Fn() + Send + 'static>>;

pub struct TrayBody {
    entries: Arc<Mutex<Vec<CallBackEntry>>>,
    info: WindowInfo,
    windows_loop: Option<thread::JoinHandle<()>>,
    event_loop: Option<thread::JoinHandle<()>>,
    event_tx: Sender<WindowsTrayEvent>,
}

impl TrayBody {
    pub fn new(title: &str, icon: &str) -> Result<Self> {
        let entries = Arc::new(Mutex::new(Vec::new()));
        let (event_tx, event_rx) = channel::<WindowsTrayEvent>();

        let entries_clone = Arc::clone(&entries);
        let event_loop = thread::spawn(move || loop {
            if let Ok(v) = event_rx.recv() {
                if v.0 == u32::MAX {
                    break;
                }

                padlock::mutex_lock(&entries_clone, |ents: &mut Vec<CallBackEntry>| match &ents
                    [v.0 as usize]
                {
                    Some(f) => f(),
                    None => (),
                })
            }
        });

        let (tx, rx) = channel();

        let event_tx_clone = event_tx.clone();
        let windows_loop = thread::spawn(move || unsafe {
            let info = match init_window() {
                Ok(info) => {
                    tx.send(Ok(info.clone())).ok();
                    info
                }

                Err(e) => {
                    tx.send(Err(e)).ok();
                    return;
                }
            };

            WININFO_STASH.with(|stash| {
                let data = WindowsLoopData {
                    info,
                    tx: event_tx_clone,
                };

                (*stash.borrow_mut()) = Some(data);
            });

            run_loop();
        });

        let info = match rx.recv().unwrap() {
            Ok(i) => i,
            Err(e) => return Err(e),
        };

        let w = Self {
            entries,
            info,
            windows_loop: Some(windows_loop),
            event_loop: Some(event_loop),
            event_tx,
        };

        w.set_tooltip(title).context("Failed to set tooltip")?;
        w.set_icon(icon).context("Failed to set icon")?;

        Ok(w)
    }

    pub fn set_icon(&self, icon: &str) -> Result<()> {
        self.set_icon_from_resource(icon)
    }

    pub fn add_label(&mut self, label: &str) -> Result<()> {
        self.add_label_with_id(label).context("Failed to add label")?;
        Ok(())
    }

    pub fn add_label_with_id(&mut self, label: &str) -> Result<u32> {
        let item_idx = padlock::mutex_lock(&self.entries, |entries| {
            let len = entries.len();
            entries.push(None);
            len
        }) as u32;

        let mut st = to_wstring(label);
        let mut item = unsafe { mem::zeroed::<MENUITEMINFOW>() };
        item.cbSize = mem::size_of::<MENUITEMINFOW>() as u32;
        item.fMask = MIIM_FTYPE | MIIM_STRING | MIIM_ID | MIIM_STATE;
        item.fType = MFT_STRING;
        item.fState = MFS_DISABLED | MFS_UNHILITE;
        item.wID = item_idx;
        item.dwTypeData = st.as_mut_ptr();
        item.cch = (label.len() * 2) as u32;

        unsafe {
            if InsertMenuItemW(self.info.hmenu, item_idx, 1, &item) == 0 {
                bail!(get_win_os_error("Error inserting menu item"));
            }
        }
        Ok(item_idx)
    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F, icon_id: Option<&str>) -> Result<()>
    where
        F: Fn() + Send + 'static,
    {
        self.add_menu_item_with_id(label, cb, icon_id).context("Failed to add menu item")?;
        Ok(())
    }

    pub fn add_menu_item_with_id<F>(&mut self, label: &str, cb: F, icon_id: Option<&str>) -> Result<u32>
    where
        F: Fn() + Send + 'static,
    {
        let item_idx = padlock::mutex_lock(&self.entries, |entries| {
            let len = entries.len();
            entries.push(Some(Box::new(cb)));
            len
        }) as u32;

        let mut st = to_wstring(label);
        let mut item = unsafe { mem::zeroed::<MENUITEMINFOW>() };
        item.cbSize = mem::size_of::<MENUITEMINFOW>() as u32;
        item.fMask = MIIM_FTYPE | MIIM_STRING | MIIM_ID | MIIM_STATE;
        item.fType = MFT_STRING;
        item.wID = item_idx;
        item.dwTypeData = st.as_mut_ptr();
        item.cch = (label.len() * 2) as u32;

        if let Some(resource_name) = icon_id {
            unsafe {
                let hicon = LoadImageW(
                    self.info.hmodule,
                    to_wstring(resource_name).as_ptr(),
                    IMAGE_ICON,
                    16,
                    16,
                    LR_DEFAULTCOLOR,
                ) as HICON;

                if hicon != 0 {
                    match icon_to_hbitmap(hicon) {
                        Ok(hbmp) => {
                            item.fMask |= MIIM_BITMAP;
                            item.hbmpItem = hbmp;
                        }
                        Err(_) => {
                            bail!("Error loading icon");
                        }
                    }
                } else {
                    bail!("Error loading icon");
                }
            }
        }

        unsafe {
            if InsertMenuItemW(self.info.hmenu, item_idx, 1, &item) == 0 {
                bail!("Error inserting menu item");
            }
        }

        Ok(item_idx)
    }

    pub fn add_separator(&mut self) -> Result<()> {
        self.add_separator_with_id().context("Failed to add separator with id")?;
        Ok(())
    }

    pub fn add_separator_with_id(&mut self) -> Result<u32> {
        let item_idx = padlock::mutex_lock(&self.entries, |entries| {
            let len = entries.len();
            entries.push(None);
            len
        }) as u32;

        let mut item = unsafe { mem::zeroed::<MENUITEMINFOW>() };
        item.cbSize = mem::size_of::<MENUITEMINFOW>() as u32;
        item.fMask = MIIM_FTYPE | MIIM_ID | MIIM_STATE;
        item.fType = MFT_SEPARATOR;
        item.wID = item_idx;

        unsafe {
            if InsertMenuItemW(self.info.hmenu, item_idx, 1, &item) == 0 {
                bail!("Error inserting menu separator");
            }
        }
        Ok(item_idx)
    }

    pub fn set_tooltip(&self, tooltip: &str) -> Result<()> {
        let wide_tooltip = to_wstring(tooltip);

        if wide_tooltip.len() > 128 {
            bail!("The tooltip may not exceed 127 wide bytes");
        }

        let mut nid = unsafe { mem::zeroed::<NOTIFYICONDATAW>() };
        nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.info.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_TIP;

        #[cfg(target_arch = "x86")]
        {
            let mut tip_data = [0u16; 128];
            tip_data[..wide_tooltip.len()].copy_from_slice(&wide_tooltip);
            nid.szTip = tip_data;
        }

        #[cfg(not(target_arch = "x86"))]
        nid.szTip[..wide_tooltip.len()].copy_from_slice(&wide_tooltip);

        unsafe {
            if Shell_NotifyIconW(NIM_MODIFY, &nid) == 0 {
                bail!("Error setting tooltip");
            }
        }
        Ok(())
    }

    fn set_icon_from_resource(&self, resource_name: &str) -> Result<()> {
        let icon = unsafe {
            let handle = LoadImageW(
                self.info.hmodule,
                to_wstring(resource_name).as_ptr(),
                IMAGE_ICON,
                64,
                64,
                LR_DEFAULTCOLOR,
            );

            if handle == 0 {
                bail!("Error setting icon from resource");
            }

            handle
        };

        self._set_icon(icon)
    }

    fn _set_icon(&self, icon: HICON) -> Result<()> {
        let mut nid = unsafe { mem::zeroed::<NOTIFYICONDATAW>() };
        nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.info.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON;
        nid.hIcon = icon;

        unsafe {
            if Shell_NotifyIconW(NIM_MODIFY, &nid) == 0 {
                bail!("Error setting icon");
            }
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        unsafe {
            PostMessageW(self.info.hwnd, WM_DESTROY, 0, 0);
        }

        if let Some(t) = self.windows_loop.take() {
            t.join().ok();
        }

        if let Some(t) = self.event_loop.take() {
            self.event_tx.send(WindowsTrayEvent(u32::MAX)).ok();
            t.join().ok();
        }
    }

    pub fn shutdown(&self) -> Result<()> {
        let mut nid = unsafe { mem::zeroed::<NOTIFYICONDATAW>() };
        nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.info.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON;

        unsafe {
            if Shell_NotifyIconW(NIM_DELETE, &nid) == 0 {
                bail!("Error deleting icon from menu");
            }
        }

        Ok(())
    }
}

impl Drop for TrayBody {
    fn drop(&mut self) {
        self.shutdown().ok();
        self.quit();
    }
}

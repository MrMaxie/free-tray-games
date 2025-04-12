<p align="center">
  <img src="assets/banner.png" alt="Free Games Tray" />
</p>

![GitHub Release](https://img.shields.io/github/v/release/MrMaxie/free-tray-games?style=for-the-badge&logo=github&label=DOWNLOAD&link=https%3A%2F%2Fgithub.com%2Ftwojuser%2Ftwojrepo%2Freleases%2Flatest)

## 🎮 What It Does

This application runs quietly in the system tray and helps you stay updated on free games available across multiple major platforms:

- ✅ Fetches a list of **currently free games** from **GOG**, **Steam**, and **Epic Games**.
- 🌐 Uses the [gamerpower.com](https://www.gamerpower.com/) public API as a data source.
- 🔁 **Auto-refreshes every 3 hours** to keep the list up to date.
- 🔄 You can **manually trigger a refresh** at any time from the tray menu.
- 🔔 **System push notifications** will appear for newly detected entries (linking directly to the free game page). _These can be disabled._
- 📋 The current list of active offers is always accessible directly from the tray menu.
- 🧹 No installation or user directory usage — **all files are located next to the executable or stored in temp**.
  Just delete the exe folder to remove it completely.

## 🪟 Platform

- **Windows-only**
- Optimized for **Windows 10 and 11**
- May not fully support earlier versions due to use of modern push notification APIs

## 🧭 Planned Features

- ➕ Add support for more providers beyond gamerpower.com
- ⏱ Allow customizing the auto-refresh interval
- ⚙️ Add whitelist/blacklist support for platforms or giveaway types
- 🧪 (Maybe) implement a custom tray menu renderer for better readability and richer display options

## 📦 License

Licensed under the **Apache 2.0** license. See [`LICENSE.rtf`](LICENSE.rtf) for details.
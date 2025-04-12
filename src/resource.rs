use std::ops::Deref;

pub enum ResourceIcon {
    Main,

    BrandEpic,
    BrandSteam,
    BrandGog,
    BrandGithub,

    NotificationsEnabled,
    NotificationsDisabled,

    Refresh,
}

impl ResourceIcon {
    pub fn get_icon_path(&self) -> &'static str {
        match self {
            ResourceIcon::Main => "main",
            ResourceIcon::BrandEpic => "tray-brand-epic-icon",
            ResourceIcon::BrandGithub => "tray-brand-github-icon",
            ResourceIcon::BrandGog => "tray-brand-gog-icon",
            ResourceIcon::BrandSteam => "tray-brand-steam-icon",
            ResourceIcon::NotificationsDisabled => "tray-notifications-disabled-icon",
            ResourceIcon::NotificationsEnabled => "tray-notifications-enabled-icon",
            ResourceIcon::Refresh => "tray-refresh-icon",
        }
    }
}

impl Deref for ResourceIcon {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.get_icon_path()
    }
}
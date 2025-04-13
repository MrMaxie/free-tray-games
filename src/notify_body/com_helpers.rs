use anyhow::Result;
use windows::{
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::ToastNotification,
};

pub fn create_toast_notification(xml: &XmlDocument) -> Result<ToastNotification> {
    Ok(ToastNotification::CreateToastNotification(xml)?)
}
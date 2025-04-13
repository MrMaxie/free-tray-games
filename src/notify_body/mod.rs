use anyhow::Result;
use windows::{
    core::HSTRING,
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::ToastNotificationManager,
};

mod com_helpers;
use com_helpers::create_toast_notification;

pub struct WinToastNotify {
    app_id: String,
    title: Option<String>,
    messages: Vec<String>,
    image: Option<String>,
    open_url: Option<String>,
}

impl WinToastNotify {
    pub fn new(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            title: None,
            messages: vec![],
            image: None,
            open_url: None,
        }
    }

    pub fn set_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn set_messages(mut self, messages: Vec<&str>) -> Self {
        self.messages = messages.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn set_image(mut self, image: &str) -> Self {
        self.image = Some(image.to_string());
        self
    }

    pub fn set_open(mut self, url: &str) -> Self {
        self.open_url = Some(url.to_string());
        self
    }

    pub fn show(&self) -> Result<()> {
        let toast_xml = XmlDocument::new()?;
        toast_xml.LoadXml(&HSTRING::from(format!(
            r#"<toast{launch}>
                <visual>
                    <binding template="ToastGeneric">
                        {image}
                        <text>{title}</text>
                        <text>{body}</text>
                    </binding>
                </visual>
            </toast>"#,
            launch = if let Some(url) = &self.open_url {
                format!(" activationType=\"protocol\" launch=\"{}\"", url)
            } else {
                "".to_string()
            },
            image = if let Some(image) = &self.image {
                format!("<image placement=\"hero\" src=\"{}\" />", image)
            } else {
                "".to_string()
            },
            title = self.title.as_deref().unwrap_or(""),
            body = self.messages.join("\\n")
        )))?;

        let toast = create_toast_notification(&toast_xml)?;
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(
            self.app_id.clone(),
        ))?;
        notifier.Show(&toast)?;

        Ok(())
    }
}

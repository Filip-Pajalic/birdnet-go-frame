//! The setup page: one form, no scripts, small enough for a phone on a slow hotspot.

use alloc::string::String;
use core::fmt::Write;

use frame_core::Rotation;

use crate::frame_settings::FrameSettings;

/// What the page shows besides the settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PageInfo<'a> {
    /// The running firmware's version.
    pub firmware: &'a str,
    /// The battery's voltage, if there is a battery.
    pub battery_millivolts: Option<u16>,
    /// Wi-Fi networks the frame saw before it opened the hotspot, offered as suggestions.
    pub networks: &'a [String],
    /// A message above the form, e.g. why the last save failed.
    pub message: Option<&'a str>,
}

/// The page's head and style, shared by both pages.
const HEAD: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>Bird frame setup</title><style>\
body{font-family:system-ui,sans-serif;max-width:28rem;margin:1.5rem auto;padding:0 1rem;\
color:#1d2a1f;background:#f6f4ee}h1{font-size:1.4rem}label{display:block;margin-top:1rem;\
font-weight:600}input,select{width:100%;box-sizing:border-box;padding:.6rem;font-size:1rem;\
margin-top:.3rem;border:1px solid #9aa59b;border-radius:.4rem;background:#fff}\
.check{font-weight:400}.check input{width:auto}button{margin-top:1.5rem;width:100%;\
padding:.8rem;font-size:1.05rem;border:0;border-radius:.4rem;background:#2f6b3a;color:#fff}\
.note{color:#56615a;font-size:.9rem}.message{background:#fbe3dc;padding:.6rem;\
border-radius:.4rem}</style></head><body>";

/// Returns the setup page with `current`'s settings filled in. The saved password is never
/// sent back; leaving the field empty keeps it.
#[must_use]
pub fn render(current: Option<&FrameSettings>, info: &PageInfo<'_>) -> String {
    let mut page: String = String::from(HEAD);
    page.push_str("<h1>Bird frame setup</h1>");
    if let Some(message) = info.message {
        page.push_str("<p class=\"message\">");
        push_escaped(&mut page, message);
        page.push_str("</p>");
    }
    page.push_str("<form method=\"post\" action=\"/save\">");
    page.push_str("<label for=\"ssid\">Wi-Fi name</label><input id=\"ssid\" name=\"ssid\" required maxlength=\"32\" list=\"networks\" value=\"");
    push_escaped(
        &mut page,
        current.map_or("", |settings| settings.ssid.as_str()),
    );
    page.push_str("\"><datalist id=\"networks\">");
    for network in info.networks {
        page.push_str("<option value=\"");
        push_escaped(&mut page, network);
        page.push_str("\">");
    }
    page.push_str("</datalist>");
    let placeholder: &str = if current.is_some_and(|settings| !settings.password.is_empty()) {
        " placeholder=\"unchanged\""
    } else {
        ""
    };
    let _ = write!(
        page,
        "<label for=\"password\">Wi-Fi password</label><input id=\"password\" name=\"password\" \
         type=\"password\" maxlength=\"63\" autocomplete=\"off\"{placeholder}>\
         <label class=\"check\"><input type=\"checkbox\" name=\"open\"> Open network, no \
         password</label>"
    );
    page.push_str("<label for=\"server\">Frame server</label><input id=\"server\" name=\"server\" required inputmode=\"decimal\" placeholder=\"192.168.1.20:8080\" value=\"");
    if let Some(settings) = current {
        let _ = write!(page, "{}", settings.server);
    }
    page.push_str(
        "\"><p class=\"note\">The address and port of the computer running the server.</p>",
    );
    page.push_str("<label for=\"frame\">Frame name</label><input id=\"frame\" name=\"frame\" maxlength=\"32\" pattern=\"[a-z0-9-]*\" placeholder=\"hall\" value=\"");
    push_escaped(
        &mut page,
        current.map_or("", |settings| settings.frame.as_str()),
    );
    page.push_str(
        "\"><p class=\"note\">Optional. Picks this frame's own layout on the server.</p>",
    );
    page.push_str(
        "<label for=\"rotation\">Rotation</label><select id=\"rotation\" name=\"rotation\">",
    );
    let chosen: Option<Rotation> = current.and_then(|settings| settings.rotation);
    let options: [(Option<Rotation>, &str); 5] = [
        (None, "As the server says"),
        (Some(Rotation::Degrees0), "0° (landscape)"),
        (Some(Rotation::Degrees90), "90° (portrait)"),
        (Some(Rotation::Degrees180), "180° (landscape, upside down)"),
        (Some(Rotation::Degrees270), "270° (portrait, upside down)"),
    ];
    for (rotation, label) in options {
        let value: String = rotation.map_or_else(
            || String::from("server"),
            |rotation| alloc::format!("{}", rotation.degrees()),
        );
        let selected: &str = if rotation == chosen { " selected" } else { "" };
        let _ = write!(page, "<option value=\"{value}\"{selected}>{label}</option>");
    }
    page.push_str("</select><button type=\"submit\">Save and restart</button></form>");
    let _ = write!(page, "<p class=\"note\">Firmware ");
    push_escaped(&mut page, info.firmware);
    if let Some(millivolts) = info.battery_millivolts {
        let _ = write!(
            page,
            " · battery {}.{:02} V",
            millivolts / 1000,
            millivolts % 1000 / 10
        );
    }
    page.push_str("</p></body></html>");
    page
}

/// Returns the page shown after a save, while the frame restarts.
#[must_use]
pub fn saved(settings: &FrameSettings) -> String {
    let mut page: String = String::from(HEAD);
    page.push_str("<h1>Saved</h1><p>The frame is restarting and will join <b>");
    push_escaped(&mut page, &settings.ssid);
    page.push_str(
        "</b>. Its hotspot closes now; the first page appears within a minute or two.</p>",
    );
    page.push_str("<p class=\"note\">To change the settings again, hold the frame's refresh button while you press reset.</p></body></html>");
    page
}

/// Appends `text` with the characters that mean something in HTML escaped.
fn push_escaped(page: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '&' => page.push_str("&amp;"),
            '<' => page.push_str("&lt;"),
            '>' => page.push_str("&gt;"),
            '"' => page.push_str("&quot;"),
            '\'' => page.push_str("&#39;"),
            _ => page.push(character),
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::vec;
    use core::net::Ipv4Addr;

    use panel_wake::FrameAddress;

    use super::*;

    #[test]
    fn fills_in_the_settings_but_never_the_password() {
        let settings: FrameSettings = FrameSettings::new(
            "Garden \"5G\"",
            "secret-password",
            FrameAddress::new(Ipv4Addr::new(192, 168, 1, 20), 8081),
            Some(Rotation::Degrees90),
            "hall",
        )
        .expect("valid");
        let networks = vec![String::from("<script>")];
        let page: String = render(
            Some(&settings),
            &PageInfo {
                firmware: "0.2.0",
                battery_millivolts: Some(3_905),
                networks: &networks,
                message: Some("The server must be an address."),
            },
        );
        assert!(page.contains("value=\"Garden &quot;5G&quot;\""));
        assert!(page.contains("value=\"192.168.1.20:8081\""));
        assert!(page.contains("<option value=\"90\" selected>"));
        assert!(page.contains("placeholder=\"unchanged\""));
        assert!(page.contains("&lt;script&gt;"));
        assert!(page.contains("battery 3.90 V"));
        assert!(page.contains("The server must be an address."));
        assert!(!page.contains("secret-password"));
        let empty: String = render(None, &PageInfo::default());
        assert!(empty.contains("<option value=\"server\" selected>"));
        assert!(saved(&settings).contains("Garden &quot;5G&quot;"));
    }
}

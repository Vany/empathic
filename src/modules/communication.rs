use serde::Deserialize;
use serde_json::json;

use crate::common::{send_error, send_response};
use crate::platform::{Platform, get_tts};

#[derive(Deserialize)]
pub struct SayArgs {
    pub text: String,
    pub voice: Option<String>,
}

pub fn say(id: u64, args: SayArgs) {
    let platform = Platform::current();
    let tts = get_tts();

    // Check TTS availability
    if !tts.is_available() {
        let install_msg = match platform {
            Platform::Linux => "Install with: sudo apt install espeak-ng (or festival)",
            Platform::Windows => "PowerShell required",
            Platform::MacOS => "Command 'say' not found",
            Platform::Unknown => "Platform not supported",
        };
        send_error(id, -3, &format!("🔊 TTS unavailable: {install_msg}"));
        return;
    }

    // Voice parameter warning for non-macOS
    let voice_warning = if args.voice.is_some() && platform != Platform::MacOS {
        "\n⚠️ Voice parameter ignored on this platform"
    } else {
        ""
    };

    match tts.speak(&args.text) {
        Ok(_) => {
            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "🔊 {} TTS Success\n📝 Text: \"{}\"{}",
                        platform.emoji(),
                        args.text,
                        voice_warning
                    )
                }]
            });
            send_response(id, result);
        }
        Err(e) => send_error(id, -3, &e),
    }
}

use std::process::Command;

/// 🌐 Platform detection and abstractions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    MacOS,
    Linux,
    Windows,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Unknown
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Platform::MacOS => "🍎",
            Platform::Linux => "🐧",
            Platform::Windows => "🪟",
            Platform::Unknown => "❓",
        }
    }
}

/// 🔊 Cross-platform TTS abstraction
pub trait TextToSpeech {
    fn speak(&self, text: &str) -> Result<(), String>;
    fn is_available(&self) -> bool;
}

pub struct MacOSTTS;
pub struct LinuxTTS;
pub struct WindowsTTS;

impl TextToSpeech for MacOSTTS {
    fn speak(&self, text: &str) -> Result<(), String> {
        Command::new("say")
            .arg(text)
            .output()
            .map_err(|e| format!("🍎 macOS TTS failed: {e}"))?;
        Ok(())
    }

    fn is_available(&self) -> bool {
        Command::new("say").arg("--version").output().is_ok()
    }
}

impl TextToSpeech for LinuxTTS {
    fn speak(&self, text: &str) -> Result<(), String> {
        // Try espeak first, then festival
        if Command::new("espeak").arg(text).output().is_ok() {
            return Ok(());
        }
        if Command::new("festival")
            .arg("--tts")
            .arg(text)
            .output()
            .is_ok()
        {
            return Ok(());
        }
        Err("🐧 No Linux TTS available (install espeak or festival)".to_string())
    }

    fn is_available(&self) -> bool {
        Command::new("espeak").arg("--version").output().is_ok()
            || Command::new("festival").arg("--version").output().is_ok()
    }
}

impl TextToSpeech for WindowsTTS {
    fn speak(&self, text: &str) -> Result<(), String> {
        Command::new("powershell")
            .args(["-Command", &format!("Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{text}')")])
            .output()
            .map_err(|e| format!("🪟 Windows TTS failed: {e}"))?;
        Ok(())
    }

    fn is_available(&self) -> bool {
        Command::new("powershell")
            .arg("-Command")
            .arg("Get-Command")
            .output()
            .is_ok()
    }
}

/// Get platform-specific TTS implementation
pub fn get_tts() -> Box<dyn TextToSpeech> {
    match Platform::current() {
        Platform::MacOS => Box::new(MacOSTTS),
        Platform::Linux => Box::new(LinuxTTS),
        Platform::Windows => Box::new(WindowsTTS),
        Platform::Unknown => Box::new(MacOSTTS), // Fallback
    }
}

/// 🔧 Platform-specific tool availability
#[derive(Debug)]
pub struct ToolCompatibility {
    pub supports_git: bool,
    pub supports_cargo: bool,
    pub supports_make: bool,
    #[allow(dead_code)]
    pub supports_tts: bool,
}

impl ToolCompatibility {
    pub fn for_current_platform() -> Self {
        let _platform = Platform::current();
        let tts = get_tts();

        Self {
            supports_git: Self::check_command("git"),
            supports_cargo: Self::check_command("cargo"),
            supports_make: Self::check_command("make"),
            supports_tts: tts.is_available(),
        }
    }

    fn check_command(cmd: &str) -> bool {
        Command::new(cmd).arg("--version").output().is_ok()
    }

    #[allow(dead_code)]
    pub fn compatibility_emoji(&self) -> String {
        let _platform = Platform::current();
        format!(
            "{} Git:{} Cargo:{} Make:{} TTS:{}",
            _platform.emoji(),
            if self.supports_git { "✅" } else { "❌" },
            if self.supports_cargo { "✅" } else { "❌" },
            if self.supports_make { "✅" } else { "❌" },
            if self.supports_tts { "✅" } else { "❌" }
        )
    }
}

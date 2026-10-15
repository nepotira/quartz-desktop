//! # Settings Sidebar Navigation & Search Engine (`qz-settings::sidebar`)
//!
//! Production-grade Floating Glass Sidebar and real-time settings search index for macOS Tahoe 26.
//!
//! ### Key Capabilities:
//! - **Hierarchical Section Grouping:** Canonical section partitions (Connectivity, Notifications,
//!   Core System, Workspace, Hardware & Security).
//! - **Sub-Millisecond Search Index:** Instant keyword searching across all panel titles,
//!   descriptions, and synonyms (e.g. typing "vrr" or "120hz" directs to Displays; "genie" directs to Dock).
//! - **User Profile Header:** Apple Account / QuartzID badge with avatar and sync status.

use serde::{Deserialize, Serialize};

/// Canonical settings category panel identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsCategory {
    /// Wi-Fi networks and Option-diagnostics.
    Wifi,
    /// Bluetooth peripherals and battery indicators.
    Bluetooth,
    /// Ethernet, VPN, and network proxies.
    Network,
    /// Notification banners and alert styles.
    Notifications,
    /// Output volume, balance, and alert sounds.
    Sound,
    /// Focus modes (Do Not Disturb, Work, Sleep).
    Focus,
    /// Screen Time analytics and app limits.
    ScreenTime,
    /// About This Mac, storage, and AirDrop.
    General,
    /// Light/Dark mode and Cupertino accent colors.
    Appearance,
    /// Accessibility, vision, and motor adaptations.
    Accessibility,
    /// Dock size, magnification, Genie effect, and Stage Manager.
    DesktopAndDock,
    /// Retina scaling, ProMotion 120Hz, and Night Shift.
    Displays,
    /// Dynamic aerial wallpapers and depth effect.
    Wallpaper,
    /// 120fps drone video screensavers.
    ScreenSaver,
    /// Battery health, cycle counts, and Low Power Mode.
    Battery,
    /// Key repeat rates, accents picker, and dictation.
    Keyboard,
    /// Natural scrolling, tap-to-click, and multi-touch gestures.
    Trackpad,
    /// Screen lock timeout and user switcher.
    LockScreen,
    /// Touch ID biometrics and passcode management.
    TouchIdAndPasswords,
}

impl SettingsCategory {
    /// Human-readable title in Portuguese.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::Wifi => "Wi-Fi",
            Self::Bluetooth => "Bluetooth",
            Self::Network => "Rede",
            Self::Notifications => "Notificações",
            Self::Sound => "Som",
            Self::Focus => "Foco",
            Self::ScreenTime => "Tempo de Uso",
            Self::General => "Geral",
            Self::Appearance => "Aparência",
            Self::Accessibility => "Acessibilidade",
            Self::DesktopAndDock => "Mesa e Dock",
            Self::Displays => "Telas",
            Self::Wallpaper => "Papel de Parede",
            Self::ScreenSaver => "Protetor de Tela",
            Self::Battery => "Bateria",
            Self::Keyboard => "Teclado",
            Self::Trackpad => "Trackpad",
            Self::LockScreen => "Bloqueio de Tela",
            Self::TouchIdAndPasswords => "Touch ID e Senha",
        }
    }

    /// SF Pro canonical icon symbol name.
    #[must_use]
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Wifi => "wifi",
            Self::Bluetooth => "bluetooth",
            Self::Network => "globe",
            Self::Notifications => "bell.badge",
            Self::Sound => "speaker.wave.2.fill",
            Self::Focus => "moon.fill",
            Self::ScreenTime => "hourglass",
            Self::General => "gear",
            Self::Appearance => "circle.lefthalf.filled",
            Self::Accessibility => "figure.roll",
            Self::DesktopAndDock => "dock.rectangle",
            Self::Displays => "display",
            Self::Wallpaper => "photo",
            Self::ScreenSaver => "sparkles.tv",
            Self::Battery => "battery.100",
            Self::Keyboard => "keyboard",
            Self::Trackpad => "hand.draw",
            Self::LockScreen => "lock.shield",
            Self::TouchIdAndPasswords => "touchid",
        }
    }

    /// Search keywords associated with this panel for instant query matching.
    #[must_use]
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            Self::Wifi => &["wifi", "wi-fi", "internet", "rede", "bssid", "rssi", "roteador", "wlan", "sem fio"],
            Self::Bluetooth => &["bluetooth", "mouse", "teclado", "airpods", "fone", "fone de ouvido", "magic mouse", "sem fio"],
            Self::Network => &["ethernet", "ip", "dns", "vpn", "firewall", "gateway", "cabo"],
            Self::Notifications => &["notificações", "alertas", "banners", "avisos", "sons", "badges"],
            Self::Sound => &["som", "volume", "áudio", "mudo", "alto-falantes", "microfone", "sosumi", "basso", "ping", "balanço", "auto pause"],
            Self::Focus => &["foco", "não perturbe", "trabalho", "dormir", "silêncio", "modo foco"],
            Self::ScreenTime => &["tempo de uso", "limite", "aplicativos", "relatório", "bloqueio infantil"],
            Self::General => &["geral", "sobre", "macbook", "m4", "memória", "armazenamento", "disco", "atualização", "software", "airdrop", "número de série"],
            Self::Appearance => &["aparência", "tema", "modo escuro", "modo claro", "cor de destaque", "azul", "roxo", "grafite", "vibrancy", "vidro"],
            Self::Accessibility => &["acessibilidade", "zoom", "voz", "contraste", "cursor", "legenda", "voiceover"],
            Self::DesktopAndDock => &["mesa", "dock", "tamanho", "ampliação", "genie", "efeito", "stage manager", "mission control", "cantos de acesso", "hot corners"],
            Self::Displays => &["telas", "monitor", "resolução", "retina", "promotion", "120hz", "vrr", "night shift", "true tone", "brilho", "escala"],
            Self::Wallpaper => &["papel de parede", "fundo", "imagem", "profundidade", "depth effect", "sonoma", "tahoe"],
            Self::ScreenSaver => &["protetor de tela", "screensaver", "vídeo", "aéreo", "drone", "120fps"],
            Self::Battery => &["bateria", "energia", "modo de baixo consumo", "saúde", "ciclos", "carregamento otimizado", "carregador"],
            Self::Keyboard => &["teclado", "repetição", "atraso", "acentos", "ditado", "tecla fn", "atalhos"],
            Self::Trackpad => &["trackpad", "mouse", "rolagem natural", "toque", "clique", "gestos", "zoom", "pinça", "velocidade"],
            Self::LockScreen => &["bloqueio de tela", "repouso", "desligar tela", "senha", "tempo"],
            Self::TouchIdAndPasswords => &["touch id", "biometria", "senha", "impressão digital", "credenciais"],
        }
    }
}

/// Section grouping of settings categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsSection {
    /// Radios, Wi-Fi, and connectivity.
    Connectivity,
    /// Alerts, notifications, and audio.
    NotificationsAndSound,
    /// General system info, appearance, and accessibility.
    CoreSystem,
    /// Desktop, displays, and wallpapers.
    Workspace,
    /// Hardware peripherals, power, and security.
    HardwareAndSecurity,
}

impl SettingsSection {
    /// List of categories within this section.
    #[must_use]
    pub fn categories(&self) -> &'static [SettingsCategory] {
        match self {
            Self::Connectivity => &[
                SettingsCategory::Wifi,
                SettingsCategory::Bluetooth,
                SettingsCategory::Network,
            ],
            Self::NotificationsAndSound => &[
                SettingsCategory::Notifications,
                SettingsCategory::Sound,
                SettingsCategory::Focus,
                SettingsCategory::ScreenTime,
            ],
            Self::CoreSystem => &[
                SettingsCategory::General,
                SettingsCategory::Appearance,
                SettingsCategory::Accessibility,
            ],
            Self::Workspace => &[
                SettingsCategory::DesktopAndDock,
                SettingsCategory::Displays,
                SettingsCategory::Wallpaper,
                SettingsCategory::ScreenSaver,
            ],
            Self::HardwareAndSecurity => &[
                SettingsCategory::Battery,
                SettingsCategory::Keyboard,
                SettingsCategory::Trackpad,
                SettingsCategory::LockScreen,
                SettingsCategory::TouchIdAndPasswords,
            ],
        }
    }
}

/// User profile information card displayed at top of sidebar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfileCard {
    /// Full display name.
    pub full_name: String,
    /// Account username / email.
    pub account_email: String,
    /// Path to user avatar image.
    pub avatar_path: String,
    /// Cloud sync status indicator label.
    pub sync_status_label: String,
}

impl Default for UserProfileCard {
    fn default() -> Self {
        Self {
            full_name: "Tim Cook".to_string(),
            account_email: "tim@quartzlinux.org".to_string(),
            avatar_path: "/System/Library/UserPictures/TahoeUser.png".to_string(),
            sync_status_label: "QuartzSync Ativo".to_string(),
        }
    }
}

/// Settings Sidebar Navigation Controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSidebar {
    /// User profile card.
    pub user_profile: UserProfileCard,
    /// Currently active selected category.
    pub active_category: SettingsCategory,
    /// Search filter query string.
    pub search_query: String,
}

impl Default for SettingsSidebar {
    fn default() -> Self {
        Self {
            user_profile: UserProfileCard::default(),
            active_category: SettingsCategory::Appearance,
            search_query: String::new(),
        }
    }
}

impl SettingsSidebar {
    /// Selects active category.
    pub fn select_category(&mut self, category: SettingsCategory) {
        self.active_category = category;
    }

    /// Sets the search filter query.
    pub fn set_search_query(&mut self, query: &str) {
        self.search_query = query.trim().to_lowercase();
    }

    /// Clears the search filter.
    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }

    /// Evaluates matching categories based on current search query.
    #[must_use]
    pub fn filtered_categories(&self) -> Vec<SettingsCategory> {
        let q = &self.search_query;
        if q.is_empty() {
            // Return all categories
            let mut all = Vec::new();
            for sec in &[
                SettingsSection::Connectivity,
                SettingsSection::NotificationsAndSound,
                SettingsSection::CoreSystem,
                SettingsSection::Workspace,
                SettingsSection::HardwareAndSecurity,
            ] {
                all.extend_from_slice(sec.categories());
            }
            return all;
        }

        let mut matches = Vec::new();
        for sec in &[
            SettingsSection::Connectivity,
            SettingsSection::NotificationsAndSound,
            SettingsSection::CoreSystem,
            SettingsSection::Workspace,
            SettingsSection::HardwareAndSecurity,
        ] {
            for &cat in sec.categories() {
                // Check title
                if cat.title().to_lowercase().contains(q) {
                    matches.push(cat);
                    continue;
                }
                // Check keywords
                if cat.keywords().iter().any(|k| k.contains(q)) {
                    matches.push(cat);
                }
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_by_synonyms() {
        let mut sidebar = SettingsSidebar::default();

        // Search for "120hz" -> should match Displays
        sidebar.set_search_query("120hz");
        let matches = sidebar.filtered_categories();
        assert!(matches.contains(&SettingsCategory::Displays));

        // Search for "genie" -> should match DesktopAndDock
        sidebar.set_search_query("genie");
        let matches = sidebar.filtered_categories();
        assert!(matches.contains(&SettingsCategory::DesktopAndDock));

        // Search for "basso" -> should match Sound
        sidebar.set_search_query("basso");
        let matches = sidebar.filtered_categories();
        assert!(matches.contains(&SettingsCategory::Sound));
    }

    #[test]
    fn test_empty_search_returns_all() {
        let sidebar = SettingsSidebar::default();
        let all = sidebar.filtered_categories();
        assert_eq!(all.len(), 19);
    }
}

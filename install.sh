#!/usr/bin/env bash
# ==============================================================================
# Script de Instalação do QuartzLinux (macOS Tahoe 26) no CachyOS / Arch Linux
# ==============================================================================

set -e

if [ "$EUID" -ne 0 ]; then
    echo "❌ Erro: Este script precisa de privilégios de superusuário para instalar componentes do sistema."
    echo "👉 Execute: sudo ./install.sh"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🍏 ==============================================================="
echo "    Instalador do QuartzLinux (macOS Tahoe 26) no CachyOS"
echo "=================================================================="

# 1. Verifica ou compila binários em modo release
if [ ! -f "target/release/qz-desktop" ] || [ ! -f "target/release/libgtk_quartz_module.so" ] || [ ! -f "target/release/libqt_quartz_theme.so" ]; then
    echo "📦 Compilando binários e módulos em modo release..."
    export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
    sudo -u "${SUDO_USER:-$USER}" cargo build --workspace --release
fi

# 2. Instalação dos executáveis do Compositor e da Sessão
echo "🚀 Instalando executáveis em /usr/local/bin/..."
install -Dm755 "target/release/qz-desktop" "/usr/local/bin/qz-desktop"
install -Dm755 "quartz-session" "/usr/local/bin/quartz-session"
install -Dm755 "crates/qz-shell/src/qz_shell_ui.py" "/usr/local/bin/qz-shell-ui"

# 3. Instalação da entrada de sessão Wayland para plasmalogin / SDDM / GDM
echo "🖥️  Instalando descritor de sessão Wayland em /usr/share/wayland-sessions/..."
install -Dm644 "quartz.desktop" "/usr/share/wayland-sessions/quartz.desktop"

# 4. Instalação do módulo GTK3 / GTK4
echo "🎨 Instalando módulo de estilo Cocoa GTK (Traffic Lights & Liquid Glass)..."
mkdir -p "/usr/lib/gtk-3.0/modules"
install -Dm755 "target/release/libgtk_quartz_module.so" "/usr/lib/gtk-3.0/modules/libgtk_quartz_module.so"

if [ -d "/usr/lib/gtk-4.0" ]; then
    mkdir -p "/usr/lib/gtk-4.0/modules"
    install -Dm755 "target/release/libgtk_quartz_module.so" "/usr/lib/gtk-4.0/modules/libgtk_quartz_module.so"
fi

# 5. Instalação do plugin QPlatformTheme Qt5 / Qt6
echo "🧩 Instalando QPlatformTheme Qt5/Qt6..."
if [ -d "/usr/lib/qt/plugins" ]; then
    mkdir -p "/usr/lib/qt/plugins/platformthemes"
    install -Dm755 "target/release/libqt_quartz_theme.so" "/usr/lib/qt/plugins/platformthemes/libqt_quartz_theme.so"
fi

if [ -d "/usr/lib/qt6/plugins" ]; then
    mkdir -p "/usr/lib/qt6/plugins/platformthemes"
    install -Dm755 "target/release/libqt_quartz_theme.so" "/usr/lib/qt6/plugins/platformthemes/libqt_quartz_theme.so"
fi

echo ""
echo "✅ ==============================================================="
echo "    QuartzLinux (macOS Tahoe 26) instalado com sucesso!"
echo "=================================================================="
echo ""
echo "📌 COMO UTILIZAR NO CACHYOS:"
echo ""
echo "1. [TESTE IMEDIATO SEM SAIR DO KDE]:"
echo "   Você pode testar agora mesmo abrindo o terminal e rodando:"
echo "   $ qz-desktop --nested"
echo "   Isso executará o compositor em uma janela sobre o seu KDE Plasma."
echo ""
echo "2. [ENTRAR NA SESSÃO COMPLETA SUBSTITUINDO O KDE]:"
echo "   - Faça Logout do KDE Plasma (Menu Iniciar > Sair / Encerrar Sessão)."
echo "   - Na tela de login (plasmalogin / SDDM), clique no seletor de sessões no canto da tela."
echo "   - Selecione 'QuartzLinux (macOS Tahoe 26)'."
echo "   - Digite sua senha e faça login!"
echo ""
echo "3. [PARA DEFINIR COMO SESSÃO PADRÃO PERMANENTE]:"
echo "   Ao fazer login, marque a opção de lembrar sessão no seu gerenciador de login."
echo "   O KDE permanecerá instalado e seguro como fallback caso precise."
echo ""

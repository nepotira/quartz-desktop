#!/usr/bin/env bash
# ==============================================================================
# Script de Desinstalação do QuartzLinux (macOS Tahoe 26) no CachyOS / Arch Linux
# ==============================================================================

set -e

if [ "$EUID" -ne 0 ]; then
    echo "❌ Erro: Este script precisa de privilégios de superusuário para desinstalar componentes do sistema."
    echo "👉 Execute: sudo ./uninstall.sh"
    exit 1
fi

echo "🗑️  Removendo arquivos do QuartzLinux..."

rm -f "/usr/local/bin/qz-desktop"
rm -f "/usr/local/bin/quartz-session"
rm -f "/usr/share/wayland-sessions/quartz.desktop"
rm -f "/usr/lib/gtk-3.0/modules/libgtk_quartz_module.so"
rm -f "/usr/lib/gtk-4.0/modules/libgtk_quartz_module.so"
rm -f "/usr/lib/qt/plugins/platformthemes/libqt_quartz_theme.so"
rm -f "/usr/lib/qt6/plugins/platformthemes/libqt_quartz_theme.so"

echo "✅ QuartzLinux desinstalado com sucesso. Sua sessão padrão retornará ao KDE Plasma."

# QuartzLinux (`qz-desktop`)

[![CI](https://github.com/nepotira/quartz-desktop/actions/workflows/ci.yml/badge.svg)](https://github.com/nepotira/quartz-desktop/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust: 2024 (1.85+)](https://img.shields.io/badge/Rust-2024%20(1.85%2B)-orange.svg)](https://www.rust-lang.org)
[![Platform: Wayland Linux](https://img.shields.io/badge/Platform-Wayland%20Linux-green.svg)](https://wayland.freedesktop.org)

**QuartzLinux (`qz-desktop`)** é um Ambiente de Desktop e Compositor Wayland nativo de nível de produção construído do zero em **Rust (Edition 2024)** para ser visualmente, matematicamente, fisicamente e funcionalmente indistinguível do **macOS Tahoe 26**.

---

## 🏛️ Arquitetura do Monorepo

```text
quartz-desktop/
├── Cargo.toml                             # Workspace root (Edition 2024, LTO = "thin", codegen-units = 1)
├── CHANGELOG.md                           # Histórico de alterações (Keep a Changelog + SemVer)
├── docs/
│   └── ENGINEERING_STANDARDS.md           # Diretrizes estritas de documentação, branching e versionamento
├── protocol/                              # Protocolos Wayland customizados (XML)
│   ├── quartz-shell-unstable-v1.xml       # Liquid Glass, Floating Pills, Traffic Lights, Dirty State, Sheets
│   ├── quartz-global-menu-v1.xml          # Global Menu Bar, Option-dynamic items, Help Search Rector
│   └── quartz-workspace-ipc-v1.xml        # IPC para Screenshot com Alpha+Shadow, Quick Look, PiP e Reveal
├── crates/
│   ├── qz-animation/                      # Solver analítico de EDO (Spring Physics C2), Rubber-banding, Shake detector
│   ├── qz-renderer/                       # Vulkan 1.3 / GLES 3.2: Liquid Glass Refractivo, Genie 64x64, Squircle G2, Sombras erf
│   ├── qz-compositor/                     # Smithay Core, KMS/VRR ProMotion, Retina Scaling, Smart Keymapper, Click-Through
│   ├── qz-shell/                          # Top Menu Bar (Notch, Privacy Capsule), Control Center, Notifications, Dock
│   ├── qz-spotlight/                      # 4-Mode Tahoe 26 Launcher: Universal Search, Actions, Clipboard History, App Library
│   ├── qz-finder/                         # Desktop Manager & Stacks, 4 Views, Undo/Redo FS, Spring-Loaded Folders
│   ├── qz-quicklook/                      # Spacebar Preview Daemon: 120fps Rect Expansion, Markup, Media Trim
│   ├── qz-screenshot/                     # Cmd+Shift+3/4/5, Window Alpha+Shadow capture, PipeWire Screen Recorder
│   ├── qz-daemons/                        # Audio (Auto-pause), Wi-Fi/BT Option-Diagnostics, Power/Clamshell, OSDs
│   ├── qz-session-lock/                   # Zero-flicker Boot, Tahoe 26 Depth Lock Screen, 3D Cube Fast User Switch
│   ├── qz-toolkit/                        # Framework UI declarativo (AquaUI): Floating Glass Pills, Sheets, Cocoa Text Engine
│   └── apps/
│       ├── qz-settings/                   # Ajustes do Sistema Tahoe 26 (Floating Glass Sidebar)
│       ├── qz-preview/                    # Pré-Visualização completo (PDFs, Imagens, Assinatura via Trackpad)
│       ├── qz-activity-monitor/           # Monitor de Atividade (CPU, Pressão de Memória, Energia, Disco, Rede)
│       └── qz-terminal/                   # Emulador de Terminal GPU-accelerated em Liquid Glass com Drag-Drop
└── compat/
    ├── xdg-desktop-portal-quartz/         # Portal nativo: FileChooser em Sheet modal com Drag-to-Retarget
    ├── gtk-quartz-module/                 # Módulo C/Rust (LD_PRELOAD) + CSS para GTK3/GTK4/Libadwaita
    └── qt-quartz-theme/                   # Plugin QPlatformTheme + QStyle para Qt5/Qt6
```

---

## ⚡ Fundamentos Matemáticos e Físicos

### 1. Solver Analítico Fechado de EDO de 2ª Ordem ($C^2$ Contínuo)
Toda animação de janela, transição de Spaces ou arrasto elástico resolve analiticamente a equação do oscilador harmônico amortecido:
$$m \frac{d^2x}{dt^2} + c \frac{dx}{dt} + k (x - x_{\text{target}}) = 0$$

- **Regime Sub-amortecido ($\zeta < 1.0$, onde $\omega_d = \omega_0 \sqrt{1 - \zeta^2}$):**
  $$x(t) = x_{\text{target}} + e^{-\zeta \omega_0 t} \left( x_0 \cos(\omega_d t) + \frac{v_0 + \zeta \omega_0 x_0}{\omega_d} \sin(\omega_d t) \right)$$
- **Regime Criticamente Amortecido ($\zeta = 1.0$):**
  $$x(t) = x_{\text{target}} + \big(x_0 + (v_0 + \omega_0 x_0) t\big) e^{-\omega_0 t}$$
- **Redirecionamento em Voo ($C^2$ Contínuo):** Preserva a posição instantânea $x(t)$ e a velocidade $v(t)$ herdada no exato momento da alteração do destino, garantindo continuidade matemática na aceleração sem solavancos perceptíveis.

### 2. Estética "Liquid Glass" e Superelipse $G^2$ (macOS Tahoe 26)
- **Squircle com Norma $L^4$:** Borda contínua em curvatura $G^2$ com fórmula de distância com sinal (SDF):
  $$sd(p, b, r) = \sqrt[4]{q_x^4 + q_y^4} + \min(\max(q_x, q_y), 0) - r \quad (\text{onde } q = |p| - (b - r))$$
- **Perfil de Lente Refrativa:** Deslocamento refrativo com dispersão cromática radial e de borda acentuando as cores do papel de parede.
- **Vibrancy e Saturação:** Saturação de luminância Rec.709 ($S=1.85$) com blend adaptativo *Color Burn* no Modo Escuro e *Color Dodge* suave no Modo Claro.
- **Specular Rim Highlight:** Borda especular de 1px sensível à direção da iluminação virtual e à luminância do fundo.

### 3. Sombras Analíticas de Dupla Camada (Apple Dual-Layer erf)
Integração gaussiana analítica calculada via aproximação contínua da função de erro ($\text{erf}$):
- **Janela Ativa:** Camada de Oclusão de Ambiente ($\sigma=4.0\text{px}, y=2.0\text{px}$) + Penumbra Profunda atmosférica ($\sigma=38.0\text{px}, y=22.0\text{px}$).
- **Floating Glass Pills:** Sombra suave flutuante ($\sigma=10.0\text{px}, y=4.0\text{px}$) sobre o conteúdo rolante de barras laterais e toolbars.

### 4. Efeito Gênio (Genie Effect) em Malha $64 \times 64$
- Deformação não-linear de malha de 4225 vértices guiada por curvas suaves de Hermite para puxar a janela de forma fluida em direção ao ícone no Dock.

---

## 🎯 Interações Nativas do macOS Tahoe 26

- **Click-Through com a Tecla `Cmd`:** Segurar `Cmd` permite interagir diretamente com botões, controles, sliders e semáforos de janelas em segundo plano (inativas) sem trazê-las para a frente (preserva Z-order) e sem roubar o foco de teclado.
- **Scroll Desfocado:** A rolagem de ponteiro (`wl_pointer::axis`) é entregue imediatamente à superfície sob o mouse, sem alterar a janela ativa.
- **Smart Keymapper:** Inverte `Cmd` $\leftrightarrow$ `Ctrl` em aplicativos comuns com atalhos Cocoa/Emacs (`Ctrl+A/E/K/Y`), mas **preserva estritamente os sinais de hardware POSIX (`Ctrl+C` = SIGINT, `Ctrl+Z` = SIGTSTP, `Ctrl+D` = EOF)** em emuladores de terminal (`qz-terminal`, `Alacritty`, `Kitty`, etc.).
- **Semáforos (Traffic Lights):**
  - Hover conjunto revela os símbolos internos nos 3 botões simultaneamente.
  - Ponto escuro central de dirty state no botão vermelho quando `set_document_edited(true)`.
  - Hover de 500ms no botão verde abre o popover nativo de Window Tiling (divisão em metades, quadrantes ou tela cheia).
- **ProMotion VRR:** Frame pacing dinâmico escalando automaticamente de 10Hz/24Hz (repouso) para 120Hz (8.33ms budget) sob qualquer animação de mola ou gesto ativo.

---

## 🛠️ Compilação e Testes

### Compilação Local
```bash
# Executa todos os testes unitários do workspace
cargo test --workspace

# Compilação em modo release otimizado (LTO thin, 1 codegen-unit)
cargo build --workspace --release
```

### Compilação Remota em Nuvem (GitHub Actions)
O repositório está integrado ao GitHub Actions. Toda alteração enviada (`push` ou `pull request`) é automaticamente compilada, testada e validada em runners Ubuntu dedicados:
- [Acompanhar builds no GitHub Actions](https://github.com/nepotira/quartz-desktop/actions)

---

## 📜 Governança e Diretrizes de Engenharia
Todas as contribuições e alterações no código seguem os padrões estabelecidos em:
- [Diretrizes de Engenharia, Versionamento e Documentação (`docs/ENGINEERING_STANDARDS.md`)](file:///home/nepo/Downloads/MacTux/docs/ENGINEERING_STANDARDS.md)
- [Registro de Alterações (`CHANGELOG.md`)](file:///home/nepo/Downloads/MacTux/CHANGELOG.md)

---

## 📄 Licença
Distribuído sob os termos da licença **MIT** ou **Apache-2.0**.

# Registro de Alterações (Changelog)

Todas as alterações notáveis neste projeto serão documentadas neste arquivo.

O formato baseia-se no [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Semantic Versioning (SemVer 2.0.0)](https://semver.org/lang/pt-BR/).

---

## [0.19.1-alpha.1] - 2026-10-15

### Fixed & Added (Apresentação Gráfica Interativa Tahoe 26 & Integração com Sessão de Desktop)
- **Executável do Compositor (`crates/qz-compositor/src/bin/qz-desktop.rs`)**:
  - Remoção da limitação estática de 50 quadros (`tick_counter >= 50`) que encerrava a execução em 400ms no modo aninhado (`--nested`).
  - Integração do inicializador dinâmico de interface gráfica: o binário do compositor agora detecta e inicia a interface de desktop do macOS Tahoe 26 (`qz_shell_ui.py` / `/usr/local/bin/qz-shell-ui`), sincronizando o loop ProMotion a 120Hz com a vida útil da janela gráfica.
- **Ambiente de Desktop Interativo GTK4 + Cairo (`crates/qz-shell/src/qz_shell_ui.py`)**:
  - Wallpaper dinâmico em gradiente Liquid Mesh Tahoe 26 (tons profundos de azul oceano, roxo meia-noite e arcos cromáticos luminosos).
  - Top Menu Bar cravada em 28px de altura com efeito Liquid Glass, logotipo Apple ``, menus da aplicação ativa (`Finder`, `Arquivo`, `Editar`, `Visualizar`, `Ir`, `Janela`, `Ajuda`), status tray no canto direito com nível de bateria, Wi-Fi, ícone do Spotlight, alternador da Central de Controle e relógio dinâmico atualizado a cada segundo.
  - Janela de teste interativa do **Finder** com semáforos fiéis às especificações da Apple (glifos internos `×`, `−`, `⤢` revelados no hover do grupo, escurecimento de luminância de 18% no clique ativo, barra lateral descolada em pílula flutuante e grade de pastas).
  - Janela de terminal GPU Liquid Glass translúcida executando comandos (`neofetch`, `uname -a`, `clear`, etc.).
  - **Dock em Vidro Líquido com Magnificação Parabólica Real**: ampliação suave de ícones baseada em curva cossenoidal/gaussiana conforme a proximidade do cursor, pontos indicadores de apps em execução e animação elástica de salto (bounce).
  - **Spotlight Search (`Cmd+Space`)**: barra de pesquisa flutuante centralizada a 20% do topo da tela com cantos squircle $R=16\text{px}$, filtragem em tempo real e lançamento de aplicativos.
  - **Central de Controle**: painel deslizante em Liquid Glass com cartões de conectividade (Wi-Fi, Bluetooth), modo Foco e controles deslizantes de Brilho da Tela e Volume do Som.
- **Instalador do Sistema (`install.sh`, `uninstall.sh`)**:
  - Adição da instalação e remoção automática do executável `/usr/local/bin/qz-shell-ui` para integração transparente com `quartz-session` e com o gerenciador de login do CachyOS.

---

## [0.19.0-alpha.1] - 2026-10-15

### Fixed & Refactored (Auditoria Forense de Fidelidade Visual, Física e Cromática macOS Tahoe 26)
- **Cores & Luzes de Tráfego (`compat/gtk-quartz-module`, `crates/qz-compositor`)**:
  - Correção cromática exata dos semáforos: Fechar (`#FF5F56`, borda `#E0443E`, glifo `#4D0000`, dirty dot `#4A0002`), Minimizar (`#FFBD2E`, borda `#DEA123`, glifo `#995700`), Zoom/Fullscreen (`#27C93F`, borda `#1AAB29`, glifo `#006500` com alternância dinâmica para `+` ao pressionar `Option`).
  - Iluminação conjunta em grupo ao passar o cursor sobre a caixa delimitadora dos 3 botões em janelas inativas, com desaturação translúcida monocromática no estado ocioso (`rgba(128, 128, 128, 0.24)` no Dark Mode / `rgba(0, 0, 0, 0.15)` no Light Mode).
  - Escurecimento físico de luminância de 18% no botão pressionado durante o evento de mouse-down.
- **Cinemática & Física do Dock (`crates/qz-animation`, `crates/qz-shell`)**:
  - Adição do envelope de entrada e saída por mola analítica (`SpringParams::dock_hover_envelope()`, $m=1.0, k=420.0, c=40.0$) modulando suavemente a ampliação de ícones sem saltos pontuais.
  - Atualização das métricas canônicas do Dock Tahoe 26: tamanho base $S_{\text{base}} = 64\text{px}$, tamanho máximo $S_{\text{max}} = 128\text{px}$, raio de alcance gaussiano $\sigma = 180\text{px}$.
  - Adição de suporte universal à dilatação temporal Shift Slow-Motion ($0.15\times$, $k \to 0.0225 \cdot k, c \to 0.15 \cdot c$).
- **Equação de Frustração na Lock Screen (`crates/qz-session-lock`)**:
  - Correção da oscilação analítica de erro de senha para a equação canônica do macOS Tahoe 26: $\Delta x(t) = 14 \cdot e^{-8t} \cdot \sin(38t)$ (início suave contínuo em 0 com 4 ciclos de rejeição amortecida em $\approx 0.45\text{s}$).
- **Menu Bar Double-Flash (`crates/qz-shell`)**:
  - Implementação da máquina de estados `MenuFlashAnimation`: 2 ciclos de piscar/inversão em 120ms (0..30ms ativo, 30..60ms inativo, 60..90ms ativo, 90..120ms inativo) antes de despachar a ação e recolher o menu.
- **Window Tiling Popover (`crates/qz-compositor`)**:
  - Criação do módulo `tiling.rs` gerenciando o popover flutuante ativado após 500ms de hover sobre o botão verde: Tela Cheia, Dividir Esquerda/Direita, 4 Quadrantes, Preencher (Fill) e Centralizar (Center), com margens canônicas de 8px em relação às bordas e entre janelas adjacentes.
- **Pílula de Vidro Flutuante (`crates/qz-toolkit`, `crates/apps/qz-settings`)**:
  - Implementação de `FloatingGlassPill` com margens internas (`inset_margin = 8.0px`, top, bottom, left), raio Squircle $G^2$ de 12px e sombra analítica interna ($\sigma = 10\text{px}, \Delta y = 4\text{px}, \alpha = 0.22$) com conteúdo rolando por baixo com Liquid Glass backdrop blur.

---

## [0.18.0-alpha.1] - 2026-10-15

### Added
- **Crate `qt-quartz-theme` (Fase 18 - Plugin QPlatformTheme + QStyle para Qt5/Qt6: Luzes de Tráfego, QPalette Cupertino e Métricas de Widgets Cocoa)**:
  - **Mapeamento de Cores e Roles QPalette (`src/palette.rs`)**:
    - Suporte a todos os papéis canônicos de cor (`Window`, `WindowText`, `Base`, `Text`, `AlternateBase`, `Button`, `ButtonText`, `Highlight`, `HighlightedText`, `ToolTipBase`, `ToolTipText`, `Mid`).
    - Decomposição estrita nos 3 grupos de estado do Qt (`Active`, `Inactive`, `Disabled`).
    - Injeção dinâmica da cor de destaque do usuário (`Highlight`) com desativação sutil em janelas em segundo plano.
  - **Decoração de Janela e Luzes de Tráfego (`src/decorations.rs`)**:
    - Posicionamento métrico das luzes de tráfego alinhadas à esquerda: Fechar ($x=12$), Minimizar ($x=32$) e Maximizar/Zoom ($x=52$) com diâmetro de 12px.
    - Hit-testing circular de alta precisão para disparar ações de fechar, minimizar ou maximizar.
    - Cálculo de centralização geométrica do título da janela na barra de título.
  - **Motor de Métricas e Estilo QStyle Tahoe 26 (`src/style.rs`)**:
    - Métricas exatas (`pixel_metric`): raio de canto de botões (6px/8px), indicadores de Checkbox e RadioButton (14x14px), espessura de barras de rolagem (12px hover, 6px ociosa sem setas de rolagem), e diâmetro de thumb de controles deslizantes (18px).
    - Diretrizes de comportamento (`style_hint`): barras de rolagem flutuantes transitórias ativadas (`ScrollBarTransient = true`), supressão de sublinhados em atalhos de menu (`UnderlineShortcut = false`) e depressão física de botões ao clique.
    - Cálculo de retângulos geométricos de sub-elementos para caixas de seleção, botões de opção, sliders e barras de rolagem.
  - **Pontos de Entrada C-ABI FFI (`src/lib.rs`)**:
    - Exportação de símbolos de carregamento dinâmico `qt_plugin_instance` e `qt_platform_theme_create` sob `#[unsafe(no_mangle)]` para integração direta em Qt5 e Qt6.

---

## [0.17.0-alpha.1] - 2026-10-15

### Added
- **Crate `gtk-quartz-module` (Fase 17 - Módulo de Compatibilidade GTK3/GTK4/Libadwaita: Luzes de Tráfego, Barra Lateral Floating Glass e Estilização Cocoa)**:
  - **Decoração CSD e Luzes de Tráfego Canônicas (`src/decorations.rs`)**:
    - Forçamento do layout `gtk-decoration-layout` para `"close,minimize,maximize:"` alinhado à esquerda no padrão macOS.
    - Especificação de cores exatas de Cupertino para botões de Fechar (`#FF5F56`), Minimizar (`#FFBD2E`) e Maximizar/Zoom (`#27C93F`), com glifos interativos ao passar o cursor (`✕`, `−`, `+`) e desaturação suave no estado inativo.
    - Centralização geométrica do título da janela na Headerbar compensando assimetria entre botões de ação à esquerda e à direita.
  - **Motor Gerador de CSS Dinâmico Tahoe 26 (`src/css.rs`)**:
    - Geração de folhas de estilo completas para GTK3 e GTK4/Libadwaita com suporte a modos Claro e Escuro.
    - Injeção dinâmica da cor de destaque do usuário (`@define-color accent_color`).
    - Barra lateral flutuante de vidro translúcido (`.sidebar`, `splitview > navigation`, `navigation-sidebar`) com `backdrop-filter: blur(20px)` e pílulas arredondadas de seleção ativa.
    - Botões e controles com curvatura Squircle $G^2$, borda especular superior e depressão física ao clique.
    - Campos de texto com anéis de foco resplandecentes em azul Cupertino (`box-shadow: 0 0 0 3px alpha(@accent_color, 0.35)`).
    - Barras de rolagem sobrepostas minimalistas estilo macOS (ocultas quando ociosas, pílula arredondada sem setas ao rolar).
  - **Pontos de Entrada C-ABI FFI (`src/lib.rs`)**:
    - Exportação de `gtk_module_init` e `gtk_module_display_init` sob `#[unsafe(no_mangle)]` para carregamento via `GTK_MODULES` e `LD_PRELOAD`.

---

## [0.16.0-alpha.1] - 2026-10-15

### Added
- **Crate `xdg-desktop-portal-quartz` (Fase 16 - Portal Nativo XDG para QuartzLinux: FileChooser em Sheet Modal com Drag-to-Retarget, QuickLook embutido, Screenshot, Settings e ScreenCast)**:
  - **Seletor de Arquivos em Sheet Modal Ancorada (`src/file_chooser.rs`)**:
    - Apresentação em Sheet modal caindo da barra de título da janela pai (`parent_window_id`), com escurecimento translúcido por scrim e física de molas.
    - **Drag-to-Retarget**: Arrasto de arquivos ou diretórios do Finder sobre a barra de navegação de caminho altera instantaneamente a pasta ativa do seletor.
    - **Visualização Rápida Quick Look Embutida**: Pressionamento da barra de espaço (`Spacebar`) sobre qualquer item selecionado expande um cartão flutuante de pré-visualização interativa a 120fps sem bloquear ou fechar o diálogo.
    - Filtros por tipos MIME e padrões glob (`*.pdf`, `image/*`).
  - **Portal de Captura de Tela e Pipeta de Cores (`src/screenshot.rs`)**:
    - Captura em tela cheia, seleção de área retangular com cursor em cruz, e captura de janela isolada com preservação de canal alfa e sombra suave de contato dual-layer erf.
    - Medidor Digital de Cores (`PickColor`): lupa ampliada de $8\times$ com retorno em formato RGB normalizado e hexadecimal (`#007AFF`).
  - **Sincronização de Ajustes e Aparência (`src/settings.rs`)**:
    - Implementação de `org.freedesktop.appearance`: preferência de esquema de cores (`color-scheme` 1: prefer-dark, 2: prefer-light), cor de destaque (`accent-color` sincronizada com as 8 cores canônicas de Cupertino) e contraste.
    - Emissão de sinais de alteração em tempo real para aplicativos Flatpak/XDG em execução.
  - **Transmissão de Tela ScreenCast via PipeWire (`src/screencast.rs`)**:
    - Negociação de fluxos de vídeo por monitor DRM ou por superfície de janela a 60Hz ou 120Hz ProMotion VRR.
    - Modos de cursor configuráveis: embutido no quadro (`Embedded`), oculto (`Hidden`) ou transmitido como metadados (`Metadata`).
  - **Serviço Central e Configuração FreeDesktop (`src/service.rs`)**:
    - Gerador da configuração canônica `quartz-portals.conf` definindo as rotas prioritárias de portal para o ambiente QuartzLinux.

---

## [0.15.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-terminal` (Fase 15 - Emulador de Terminal GPU-accelerated em Liquid Glass com Drag-Drop Path Escaping)**:
  - **Grade de Células de Alta Performance e Parser ANSI/VT100 (`src/grid.rs`)**:
    - Suporte a Truecolor 24-bits (`\x1b[38;2;r;g;bm` e `\x1b[48;2;r;g;bm`), paleta de 16 cores padrão ANSI, e atributos de texto completos (negrito, fraco, itálico, sublinhado, tachado, invertido e piscante).
    - Histórico de rolagem (Scrollback) profundo de 10.000 linhas com suporte a rolagem suave contínua subpixel e Cocoa rubber-banding.
    - Estilos visuais de cursor configuráveis: Bloco (`█`), Sublinhado (`_`) e Traço Vertical I-Beam (`|`).
  - **Motor de Escape de Caminhos por Arrasto e Soltura (Drag-Drop Path Escaping) (`src/dnd.rs`)**:
    - Escapamento automático de espaços e metacaracteres de shell (` `, `\`, `"`, `'`, `(`, `)`, `[`, `]`, `{`, `}`, `&`, `|`, `;`, `$`, `*`, `?`, `<`, `>`, `^`, `#`, `!`) ao arrastar arquivos/pastas do Finder ou Mesa.
    - Estratégias configuráveis: Escapamento por barras invertidas (estilo nativo macOS Terminal) e envoltório em aspas simples POSIX (`'...'`).
    - Decodificação de octetos percent-encoded (`%20` $\to$ espaço) em listas de URIs (`file://` / `text/uri-list`) do Wayland.
    - Concatenação automática de múltiplos caminhos separados por espaço prontos para execução no prompt.
  - **Gestor de Sessões PTY e Processos Shell (`src/pty.rs`)**:
    - Ciclo de vida de pseudo-terminais (PTY master/slave) com detecção automática do shell padrão do usuário (`$SHELL`, `/bin/zsh`, `/bin/bash`).
    - Injeção das variáveis de ambiente canônicas do sistema: `TERM=xterm-256color`, `COLORTERM=truecolor`, `QUARTZ_DESKTOP=1` e `LANG=en_US.UTF-8`.
    - Sinalização de redimensionamento dinâmico de colunas e linhas (`TIOCSWINSZ` / `SIGWINCH`).
  - **Perfis Visuais e Material Óptico Liquid Glass (`src/theme.rs`)**:
    - 5 perfis canônicos: `Tahoe Glass` (Acrílico translúcido escuro `#171A21` a 76% com cursor ciano), `Pro` (Vidro preto profundo com cursor verde), `Silver` (Vidro claro translúcido com tipografia escura), `Homebrew` (Verde fósforo `#33FF00` estilo CRT sobre preto puro) e `Ocean` (Acrílico azul marinho profundo com cursor âmbar).
    - Material óptico com blur Dual-Kawase de fundo ($\sigma = 24.0\text{px}$) e refração ($\eta = 1.52$).
    - Ajuste de escala tipográfica com atalhos de teclado (`Cmd++`, `Cmd+-`, `Cmd+0`).
  - **Controlador de Janela e Sessões com Abas e Divisão de Painéis (`src/window.rs`)**:
    - Arquitetura multi-abas com sessões PTY e buffers de grade isolados por aba.
    - Integração de arrasto e soltura direcionando caminhos escapados diretamente para o PTY da aba ativa.
    - Barra de busca interna no terminal (`Cmd+F`) com realce de correspondências.

---

## [0.14.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-activity-monitor` (Fase 14 - Aplicativo Monitor de Atividade Tahoe 26: CPU, Gráfico de Pressão de Memória, Energia, Disco, Rede e Force Quit)**:
  - **Telemetria de Processos e Gestão Force Quit (`src/process.rs`)**:
    - Monitoramento de PID, nome do executável, usuário, CPU % suavizado, tempo de CPU acumulado, contagem de threads, acordadas em repouso (idle wakeups), GPU %, memória RSS, memória comprimida, impacto energético, energia de 12 horas, estado App Nap, taxas de disco e rede, e arquitetura de binário (Apple Silicon nativo ARM64, Intel via Rosetta 2 e Linux nativo).
    - Ordenação dinâmica multi-colunas e filtragem avançada (Todos os Processos, Meus Processos, Processos Ativos, Aplicativos com Janelas e busca instantânea por substring ou PID).
    - Término de processos via sinais POSIX: `SIGTERM` (15, encerramento gracioso) e `SIGKILL` (9, encerramento forçado) com proteção estrita para processos críticos do sistema (PID 1).
  - **Gráfico de Pressão de Memória e Paging Tahoe 26 (`src/memory.rs`)**:
    - Classificação canônica dos 3 estados de pressão: `Normal` (Verde Cupertino `#34C759`, $<60\%$), `Aviso` (Amarelo Cupertino `#FF9500`, $60\%..=80\%$) e `Crítico` (Vermelho Cupertino `#FF3B30`, $>80\%$).
    - Decomposição precisa da memória: Memória Física instalada, Memória dos Aplicativos (App Memory), Memória Fixada (Wired Memory), Memória Comprimida (Compressed RAM), Arquivos em Cache e Espaço de Swap utilizado no SSD.
    - Buffer em anel histórico (Ring Buffer) de 120 amostras para renderização da curva contínua Bezier/spline em tempo real.
  - **Telemetria Multi-Core de CPU e Cargas Heterogêneas (`src/cpu.rs`)**:
    - Decomposição percentual do tempo de CPU: Usuário (`#007AFF` Azul), Sistema (`#FF3B30` Vermelho) e Ocioso (Idle).
    - Suporte a arquiteturas heterogêneas diferenciando núcleos de Alto Desempenho (P-cores) e Alta Eficiência (E-cores).
    - Médias de carga do kernel (Load Averages de 1, 5 e 15 minutos), contagem global de threads e buffer em anel de histórico da CPU.
  - **Impacto Energético e Telemetria de Bateria (`src/energy.rs`)**:
    - Cálculo analítico do índice de Impacto Energético:
      $$\text{Score} = 1.0 \times \text{CPU}_{\%} + 1.5 \times \text{GPU}_{\%} + 0.05 \times \text{Wakeups}_{\text{sec}}$$
    - Média móvel exponencial (EWMA) de energia nas últimas 12 horas.
    - Detecção de aplicativos em modo App Nap e identificação de Grandes Consumidores de Energia.
    - Telemetria de bateria: porcentagem de carga, fonte de alimentação (Bateria / Adaptador de Corrente), status de carregamento, contagem de ciclos, saúde e estimativa de tempo restante.
  - **Taxas de Transferência de Disco e Rede (`src/disk_network.rs`)**:
    - Monitoramento de I/O de Disco: taxas de leitura e gravação em bytes/s, contagem de operações (IOPS) e bytes cumulativos desde a inicialização.
    - Monitoramento de Rede: pacotes por segundo, largura de banda de download e upload em bytes/s e volumes totais transferidos.
    - Buffers temporais para gráficos em tempo real de tráfego de entrada e saída.
  - **Controlador de Janela e Cromo Liquid Glass (`src/window.rs`)**:
    - Barra de título unificada com Controle Segmentado de 5 abas canônicas: `CPU`, `Memória`, `Energia`, `Disco` e `Rede`.
    - Alinhamento automático da coluna de ordenação com base na aba ativa.
    - Folhas modais ancoradas de vidro líquido (Liquid Glass Sheets): Confirmação de Force Quit com opções `Cancelar`, `Encerrar` e `Forçar Encerramento`, e Inspetor Detalhado de Processos.
    - Painel de resumo dinâmico inferior alternando gráficos e estatísticas em tempo real conforme a aba selecionada.

---

## [0.13.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-preview` (Fase 13 - Aplicativo Pré-Visualização Tahoe 26: PDFs Multi-Páginas, Imagens, Anotações Avançadas, Assinatura via Trackpad e Perfis de Cor)**:
  - **Modelo de Documento Multi-Página e Viewport Subpixel (`src/document.rs`)**:
    - Suporte unificado para PDFs e imagens raster (PNG, JPEG, WebP, HEIC, TIFF).
    - Zoom contínuo de $0.1\times$ a $10.0\times$ com tamanho real (`Cmd+0`), ajustar à janela (`Cmd+9`) e margens seguras.
    - Rotações horária e anti-horária a cada $90^\circ$ (`Cmd+R` / `Cmd+L`), reordenação de páginas e exclusão de páginas.
    - Modos de layout de exibição: Página Única (`SinglePage`), Rolagem Contínua (`ContinuousScroll`), Duas Páginas (`TwoPages`) e Folha de Contatos (`ContactSheet`).
    - Rastreamento estrito de estado não-salvo (Dirty state) atualizando título e ícone proxy.
  - **Motor de Anotações e Markup Avançado (`src/markup.rs`)**:
    - Ferramentas completas de marcação: Seleção, Alfa Instantâneo (remoção de fundo/varinha mágica), Desenho livre (Sketch) com sensibilidade a pressão ($1.0..=16.0\text{px}$) e suavização por splines cúbicas de Catmull-Rom.
    - Reconhecimento automático de formas geométricas: Detecção analítica de círculos/elipses, retângulos e setas direcionais a partir do traçado à mão livre.
    - Formas vetoriais nativas: Retângulo, Elipse, Linha, Seta, Balão de fala, Estrela de 5 pontas e Lupa óptica (Loupe) com ampliação ajustável ($1.5\times$ a $5.0\times$).
    - Marcador de texto translúcido com mesclagem multiplicativa (Highlighter) e caixas de texto com formatação tipográfica.
    - Pilha transacional completa de Desfazer/Refazer (Undo/Redo) com operações atômicas.
  - **Captura e Cofre de Assinaturas via Trackpad (`src/signature.rs`)**:
    - Superfície de desenho no trackpad em coordenadas normalizadas $(u, v) \in [0.0, 1.0] \times [0.0, 1.0]$ com pressão $p \in [0.0, 1.0]$.
    - Interpolação cúbica de Hermite ($C^1$ contínua) com variação caligráfica da espessura do traço dependente da velocidade e pressão.
    - Cofre persistente de assinaturas digitais (`SignatureVault`) com rotulagem amigável ("Assinatura Formal", "Iniciais") e inserção proporcional preservando proporção de tela em páginas do documento.
  - **Motor de Exportação e Incorporação de Perfis de Cor (`src/export.rs`)**:
    - Conversão multi-formato de alta fidelidade (PDF, PNG, JPEG, WebP, TIFF).
    - Incorporação de perfis de cor ICC: Display P3 (ampla gama de cores), sRGB, Adobe RGB (1998) e ProPhoto RGB.
    - Achatamento opcional de anotações (Flattening) em densidades de resolução configuráveis (72 DPI tela, 144 DPI @2x Retina, 300 DPI impressão).
    - Filtros flexíveis de intervalo de páginas: Todas, Página Individual ou Intervalo Contíguo.
  - **Controlador de Janela e Cromo Liquid Glass (`src/window.rs`)**:
    - Barra de título unificada com luzes de tráfego, indicador de estado não-salvo (`•` bullet dot) e Ícone Proxy de Documento interativo com ciclo de vida de arrasto para Finder/Mail.
    - Barra de ferramentas secundária flutuante de vidro líquido (Liquid Glass) recolhível para marcação (`Cmd+Shift+A`).
    - Gaveta de barra lateral multifuncional: Miniaturas de páginas com reordenação, Índice de marcadores (TOC) e lista de anotações e notas destacadas.

---

## [0.12.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-settings` (Fase 12 - Aplicativo Ajustes do Sistema Tahoe 26: Barra Lateral de Vidro Flutuante, Busca em Tempo Real e Painéis Canônicos)**:
  - **Barra Lateral Flutuante Liquid Glass e Motor de Busca (`src/sidebar.rs`)**:
    - **5 Seções Hierárquicas Canônicas**: Conectividade (Wi-Fi, Bluetooth, Rede), Notificações e Som, Sistema Principal (Geral, Aparência, Acessibilidade), Espaço de Trabalho (Mesa e Dock, Telas, Papel de Parede, Protetor de Tela) e Hardware e Segurança (Bateria, Teclado, Trackpad, Bloqueio de Tela, Touch ID e Senhas).
    - **Motor de Busca Sub-Milissegundo**: Indexação instantânea de títulos de seções, descrições e sinônimos profundos (ex: busca por "120hz" ou "vrr" direciona para Telas; "genie" direciona para Mesa e Dock; "basso" direciona para Som; "m4" direciona para Geral).
    - **Cartão de Perfil de Usuário**: Cabeçalho de perfil Apple Account / QuartzID com avatar, nome de exibição e estado de sincronização em nuvem.
  - **Painéis de Configuração Completos (`src/panels/`)**:
    - **Aparência (`appearance.rs`)**: Modos Claro, Escuro e Automático com ciclo solar; 8 cores de destaque de Cupertino (`Azul #007AFF`, `Roxo #AF52DE`, `Rosa #FF2D55`, `Vermelho #FF3B30`, `Laranja #FF9500`, `Amarelo #FFCC00`, `Verde #34C759`, `Grafite #8E8E93` e `Multicor`); e controle de matiz de papel de parede em janelas Liquid Glass.
    - **Telas (`displays.rs`)**: Modos de escala Retina (`Texto Maior`, `Padrão @2x`, `Mais Espaço`); opções de taxa de atualização (`ProMotion 120Hz` VRR dinâmico de 10Hz a 120Hz, 60Hz, 50Hz, 48Hz); agendamento de Night Shift com temperatura de cor e True Tone adaptativo.
    - **Mesa e Dock (`dock_desktop.rs`)**: Sliders de tamanho de ícones e ampliação parabólica com hover; posições (Inferior, Esquerda, Direita); efeito de minimização Genie (malha 64x64) ou Escala; alternador de Stage Manager com visualização de itens da mesa; e configuração de 4 Cantos de Acesso (Hot Corners).
    - **Som (`sound.rs`)**: Volume mestre com curva cúbica decibel, balanço estéreo, tons de alerta canônicos (`Sosumi`, `Basso`, `Ping`, `Funk`, `Hero`, `Tink`) e alternador de auto-pause CoreAudio para fones de ouvido.
    - **Bateria e Economia de Energia (`battery.rs`)**: Políticas de ativação de Modo de Baixo Consumo (Nunca, Sempre, Apenas na Bateria, Apenas no Adaptador); telemetria de saúde da bateria (porcentagem de capacidade máxima e contagem de ciclos); e carregamento otimizado a 80%.
    - **Teclado e Trackpad (`keyboard_trackpad.rs`)**: Calibração de taxa de repetição de teclas (10Hz–60Hz) e atraso inicial (150ms–600ms); ativação do menu popover de acentos ao segurar tecla (>300ms); atalho de ditado por voz; velocidade de rastreamento do trackpad, rolagem natural e gestos multi-toque 1:1.
    - **Geral e Sobre Este Mac (`general.rs`)**: Telemetria completa do hardware (especificações do chip M4 Max, memória unificada, número de série, build do sistema); gráfico segmentado de utilização de armazenamento (Aplicativos, Desenvolvedor, Sistema, Documentos); e visibilidade do AirDrop.
  - **Controlador de Janela com Histórico de Navegação (`src/window.rs`)**:
    - Layout de dois painéis em Liquid Glass com pilha de histórico (Voltar / Avançar) preservando o estado de navegação.

---

## [0.11.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-toolkit` (Fase 11 - Framework AquaUI: Pílulas de Vidro Flutuante, Modal Sheets, Barra de Abas Nativa e Motor de Tipografia Cocoa)**:
  - **Componentes e Pílulas de Vidro Flutuante (`src/widgets.rs`)**:
    - **Geometria Squircle $G^2$ Superelipse**: Equação analítica contínua de curvatura com expoente $n = 4.0$, garantindo ausência de descontinuidades na curvatura entre bordas retas e cantos arredondados.
    - **Material Óptico Liquid Glass**: Parâmetros de refração ($\eta = 1.52$), dispersão cromática ($\Delta \eta = 0.015$), blur Dual-Kawase de fundo ($\sigma = 20.0\text{px}$) e borda especular reflexiva.
    - **Botões e Micro-interações com Física de Molas**: Depressão de 4% ao pressionar ($1.0 \to 0.96 \to 1.0$) dirigida por `SpringAnimation` com preset `dock_bounce`.
    - **Controle Segmentado Deslizante**: Pílula de seleção de vidro que desliza suavemente sob os rótulos de texto com transições elásticas.
    - **Interruptor Toggle de Vidro**: Alternador estilo macOS/iOS com movimento de polegar dirigido por molas.
  - **Modal Sheets Ancoradas (`src/sheets.rs`)**:
    - **Ancoragem na Barra de Título da Janela Pai**: Desdobramento vertical caindo diretamente da base da barra de ferramentas com mola física (`SpringParams::window_open`).
    - **Camada de Escurecimento Translúcido (Scrim)**: Esmaecimento suave do conteúdo da janela pai até $35\%$ de opacidade.
    - **Gesto Interativo de Puxar para Fechar (Drag-to-Dismiss)**: Deslocamento vertical com resistência elástica Cocoa rubber-banding; liberação além do limite de $120\text{px}$ confirma o descarte elástico da folha modal.
  - **Barra de Abas Nativa com Reordenação Flocking (`src/tabs.rs`)**:
    - **Física de Reordenação por Arrasto Flocking**: Ao arrastar uma aba, as abas vizinhas deslizam automaticamente para o lado usando física de molas (`SpringParams::dock_bounce`), abrindo espaço para inserção antes da soltura.
    - **Glifo Canônico de Estado Não-Salvo (Dirty State)**: Abas com edições não salvas exibem um ponto de bala central (`•`) no botão de fechar, que se transforma dinamicamente em um `x` ao passar o cursor (comportamento nativo AppKit).
    - **Distribuição Adaptativa de Largura**: Ajuste dinâmico entre $W_{\text{min}} = 100\text{px}$ e $W_{\text{max}} = 240\text{px}$, acomodando o botão fixo de nova aba (`+`).
    - **Pílula de Fundo Deslizante Ativa**: Indicador de vidro que acompanha a aba ativa com interpolação analítica contínua.
  - **Motor de Tipografia Cocoa CoreText (`src/text.rs`)**:
    - **Posicionamento Subpixel em Quartos de Pixel**: Quantização das origens horizontais de glifos em múltiplos de $0.25\text{px}$ ($x = \text{round}(4x)/4$), preservando o desenho caligráfico de fontes como *SF Pro* sem distorções de font hinting.
    - **Escurecimento Adaptativo de Hastes (Stem Darkening)**: Modulação adaptativa de espessura de fonte baseada na luminância do fundo ($L \in [0.0, 1.0]$), aplicando até $+15\%$ de peso em fundos brancos para compensar a erosão de subpixel bleed.
    - **Cursor Caret com Pulso Suave em Cosseno**: Modulação ótica contínua de opacidade ($I(t) = \frac{1 + \cos(2\pi f t)}{2}$) a $0.8\text{ Hz}$.
    - **Quebra de Linhas Multilinha e Hit-Testing**: Layout com word wrapping e cálculo instantâneo de índice de caractere a partir de coordenadas $(x, y)$.
  - **Motor de Layout Declarativo AquaUI (`src/layout.rs`)**:
    - Estruturas de pilha flexível `VStackLayout` e `HStackLayout` com espaçamento configurável, alinhamentos ortogonais (`Leading`, `Center`, `Trailing`, `Top`, `Bottom`) e margens de borda `EdgeInsets`.
  - **Tema Global AquaUI (`src/lib.rs`)**:
    - Definição de `AquaTheme` suportando modos `Light` e `Dark`, cor de destaque Cupertino Blue (`rgba(0, 122, 255, 1.0)`) e níveis de translucidez de janela.

---

## [0.10.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-session-lock` (Fase 10 - Bloqueio de Sessão: Zero-Flicker Boot, Tahoe 26 Depth Lock Screen, Desacelerador de Vídeo Aéreo a 120fps e Troca Rápida de Usuário em Cubo 3D)**:
  - **Tela de Bloqueio Volumétrica com Efeito de Profundidade (`src/depth_lock.rs`)**:
    - **Relógio Volumétrico com Máscara de Profundidade**: Renderização espacial dos dígitos do relógio em camada 3D atrás do sujeito destacado do papel de parede (montanhas, pessoas, monumentos).
    - **Salvaguarda de Legibilidade**: Avaliação analítica da taxa de oclusão; se mais de 35% da área dos dígitos for obscurecida pelo sujeito, o efeito de profundidade é automaticamente desativado para garantir legibilidade tipográfica.
    - **Oscilador Harmônico Amortecido para Erro de Senha**: Simulação física de vibração horizontal com decaimento exponencial ($x(t) = A e^{-\gamma t} \cos(\omega t)$) com $A = 24.0\text{px}$, amortecimento $\gamma = 12.0$ e frequência de vibração $\omega = 45.0\text{ rad/s}$ ($~7.16\text{ Hz}$).
    - **Coloração Cromática Adaptativa**: Alternância automática entre `LightGlass` e `DarkGraphite` baseada na luminância do papel de parede na região do relógio.
    - **Pílula de Senha Liquid Glass e Bloqueio Temporizado**: Máquina de estados (`Locked`, `Authenticating`, `Success`, `FailedShake`, `LockedOut` após 5 tentativas incorretas).
  - **Desacelerador de Vídeo Aéreo a 120fps (`src/aerial_decelerator.rs`)**:
    - **ProMotion 120fps Screensaver**: Reprodução contínua de filmagens aéreas em drone a 120fps com integração temporal trapezoidal.
    - **Desaceleração Contínua $C^2$ ao Desbloquear**: Ao acordar ou desbloquear o sistema, a velocidade de reprodução desacelera suavemente de $1.0\times$ para $0.0\times$ ao longo de $2.4\text{ segundos}$ usando uma curva de interpolação Hermite cúbica com derivadas nulas nas duas extremidades:
      $$u = \frac{t}{T_{\text{decel}}}, \quad \text{speed}(u) = (1 - u)^2 (1 + 2u)$$
    - **Fusão Perfeita em Papel de Parede Estático**: O quadro de vídeo exato onde a velocidade atinge zero torna-se o papel de parede estático do desktop sem cortes bruscos ou saltos visuais.
  - **Troca Rápida de Usuário em Cubo 3D (`src/cube_switch.rs`)**:
    - **Projeção em Perspectiva Espacial**: Campo de visão $\text{FOV} = 45^\circ$, distância analítica de câmera $d = \frac{1}{\tan(\text{FOV} / 2)} \approx 2.41421356$ para mapeamento sem distorção na rotação zero ($\theta = 0^\circ$).
    - **Gesto Interativo 1:1 e Limite de Confirmação a 45°**: Rastreamento contínuo de arrasto com dois dedos no trackpad; soltura além de $45^\circ$ ($\pi/4$) confirma a rotação para a face do próximo usuário ($\pm 90^\circ$), enquanto soltura aquém de $45^\circ$ retorna elasticamente à face original via `SpringAnimation` com preset `mission_control` ($\zeta = 1.00$).
    - **Matriz Model-View-Projection ($\mathbf{MVP}$)**: Geração de transformações tridimensionais compatíveis com pipelines Vulkan e OpenGL ES.
  - **Handshake de Inicialização Zero-Flicker (`src/boot_handshake.rs`)**:
    - **Preservação de Scanout DRM KMS**: Elimina o piscar preto de transição do Plymouth/UEFI para o compositor Wayland reutilizando o descritor de framebuffer do scanout ativo.
    - **Conformidade com `ext-session-lock-v1`**: Isolamento de segurança estrito capturando todos os dispositivos de entrada e gerando transição suave de crossfade alfa de $300\text{ms}$ para o desktop desbloqueado.
  - **Coordenador Central de Sessão (`src/lib.rs`)**:
    - Estrutura `SessionLockCoordinator` integrando a tela de bloqueio, desacelerador aéreo, cubo 3D e handshake KMS.

---

## [0.9.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-daemons` (Fase 9 - Daemons de Sistema: CoreAudio Auto-pause, Wi-Fi/BT Option-Diagnostics, Power/Clamshell, OSD com Passos Finos de 1/64, Seleção de Acentos, Paleta de Emoji e Ditado por Voz)**:
  - **Daemon CoreAudio e Políticas de Reprodução (`src/audio.rs`)**:
    - **Pausa Automática ao Desconectar Fones / Bluetooth**: Interceptação em tempo real de desligamento ou desconexão de fones 3.5mm ou fones Bluetooth A2DP, emitindo sinal `AutoPauseAction::PauseMedia` para proteger a privacidade e evitar ejeção de som nos alto-falantes integrados.
    - **Pausa Automática ao Suspender / Bloquear**: Emissão imediata de pausa ao entrar em modo de suspensão ou bloqueio de tela.
    - **Curva Perceptual Cúbica e Escala em Decibéis**: Mapeamento não-linear de volume perceptual ($V_{\text{amp}} = x^3$) e conversão analítica para decibéis ($\text{dB} = 20 \log_{10}(V)$ com piso de $-60\text{ dB}$).
    - **Efeitos Sonoros Canônicos do macOS Tahoe 26**: Gerador e despachante de sons de sistema (`VolumeTick`, `AlertSosumi`, `AlertBasso`, `AlertPing`, `AlertFunk`, `AlertHero`, `AlertTink`, `ScreenshotCapture`, `TrashEmpty`, `DictationStart`, `DictationStop`).
    - **Gerenciamento de Dispositivos e Perfis**: Rastreamento de alto-falantes internos, microfones, HDMI/DisplayPort e perfis de codec Bluetooth (AAC, aptX, LDAC, LC3, SBC).
  - **Monitor de Radios e Diagnósticos Avançados Option-Click (`src/network.rs`)**:
    - **Inspeção de Camada Física e RF de Wi-Fi (`WifiDiagnostics`)**: Captura em tempo real de BSSID, RSSI ($\text{dBm}$), Piso de Ruído ($\text{dBm}$), Relação Sinal-Ruído ($\text{SNR} = \text{RSSI} - \text{Noise}$), Padrão PHY (802.11a/b/g/n/ac/ax/be incluindo Wi-Fi 6, 6E e 7), Canais e Larguras de Banda (20/40/80/160/320 MHz), Taxa de Transmissão Tx (Mbps), Índice MCS, Fluxos Espaciais (NSS) e Tipo de Segurança (WPA3-Personal SAE, WPA2-Enterprise, etc.).
    - **Inspeção de Periféricos Bluetooth (`BluetoothDeviceDiagnostics`)**: Rastreamento de periféricos (AirPods, Magic Mouse, Magic Keyboard, Magic Trackpad), RSSI, Porcentagem de Qualidade de Link, Latência em milissegundos, Codec de Áudio ativo e Leitura Multi-Bateria (AirPods Fone Esquerdo, Fone Direito e Estojo de Carga).
    - **Formatação Canônica de Diagnóstico**: Formatação estruturada em pares chave-valor para exibição imediata no menu Option+Click da Menu Bar.
  - **Gerenciamento de Energia, Bateria e Modo Clamshell (`src/power.rs`)**:
    - **Máquina de Estados de Modo Clamshell Determinística**:
      - Ao fechar a tampa do laptop: entra em `ClamshellMode::ClamshellActive` se e somente se houver monitor externo conectado (`external_displays_count >= 1`) E alimentação externa conectada (`ac_connected == true`), desativando o painel interno (eDP-1) e transferindo o desktop para o monitor externo.
      - Se qualquer um dos dois requisitos faltar ao fechar a tampa (ou se o cabo de energia for desconectado com a tampa fechada), o sistema entra imediatamente em modo de suspensão (`TriggerSleep`).
    - **Telemetria e Saúde da Bateria (`BatteryTelemetry`)**:
      - Cálculo de potência instantânea em Watts ($P = V \times I$).
      - Estimativa dinâmica de tempo restante em minutos para carga e descarga.
      - Avaliação de saúde da bateria: relação entre capacidade máxima atual e capacidade de fábrica de projeto ($\text{Health} = \frac{C_{\text{max}}}{C_{\text{design}}} \times 100\%$) com recomendação de serviço para saúde $< 80\%$, e contagem de ciclos de recarga.
    - **Modo de Baixo Consumo (Low Power Mode)**: Sinalização para o compositor limitar o ProMotion de 120Hz para 60Hz e reduzir passas de blur translúcido na GPU.
  - **Bezel On-Screen Display (OSD) com Passos Finos (`src/osd.rs`)**:
    - Pílula flutuante e bezel HUD estilo macOS Tahoe 26 para Volume, Brilho de Tela e Iluminação do Teclado.
    - **Granularidade em Passos de 1/64 (64 Níveis Discretos)**:
      - Passo padrão de 1/16 (16 níveis de barra, $\Delta = 4$ sub-passos).
      - **Passo ultra-fino de 1/64 acionado via `Option + Shift + Volume/Brilho`** ($\Delta = 1$ sub-passo), provendo precisão cirúrgica de áudio e brilho mínimo.
    - **Animação de Popover com Física de Molas**: Escala e entrada dirigidas por `qz_animation::SpringAnimation` (preset `dock_bounce`).
    - **Temporizador de Descarte e Esmaecimento**: Exibição por 1.8 segundos com decaimento suave de opacidade (*alpha*).
    - **Ícones Dinâmicos Adaptativos**: Transição automática de glifos baseada no nível atual (Mudo, Baixo, Médio, Alto).
  - **Subsistema de Auxiliares de Entrada (`src/input_helpers/`)**:
    - **Seletor de Diacríticos e Acentos (`accents.rs`)**:
      - Press-and-hold ativado ao segurar tecla por mais de 300ms (`hold_threshold`), cancelando a repetição contínua e projetando popover numerado sobre o cursor de texto (ex: segurar `e` apresenta `1: é, 2: è, 3: ê, 4: ë, 5: ē, 6: ė, 7: ę`).
      - Confirmação via tecla numérica (1–9), Setas + Enter, ou clique.
    - **Paleta de Emoji e Símbolos (`emoji.rs`)**:
      - Atalho canônico `Ctrl + Cmd + Space`.
      - Catálogo completo estruturado nas 9 categorias do Tahoe 26.
      - Modificadores Unicode Fitzpatrick de tom de pele (Padrão, Claro, Médio-Claro, Médio, Médio-Escuro, Escuro).
      - Busca textual sub-milissegundo por palavras-chave e sinônimos, fila MRU de usados recentemente (até 32 itens) e favoritos fixados.
    - **Daemon de Ditado por Voz (`dictation.rs`)**:
      - Detecção de duplo-toque na tecla `Fn` (Globo) em intervalo $\le 400\text{ms}$.
      - Medidor de RMS de áudio do microfone com suavização exponencial para alimentar o indicador pulsante na Privacy Capsule da Menu Bar.
      - Buffer de transcrição com hipóteses parciais em tempo real e confirmação final com pontuação.
  - **Coordenador Unificado de Daemons (`src/lib.rs`)**:
    - Estrutura `SystemDaemonsCoordinator` centralizando todos os daemons e integrando eventos de ciclo de vida com o compositor.

---

## [0.8.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-screenshot` (Fase 8 - Captura de Tela Cmd+Shift+3/4/5, Janela Isolada com Alfa e Sombra, Gravador PipeWire e Miniatura Flutuante)**:
  - **Mecanismo de Modos e Atalhos de Captura (`src/capture/`)**:
    - `Cmd+Shift+3`: Captura instantânea de tela cheia para todos os monitores conectados.
    - `Cmd+Shift+4`: Seleção retangular interativa com retícula (*crosshair*), trava de proporção 1:1 ao segurar `Shift` e expansão central ao segurar `Option`.
    - **Modo Câmera com Barra de Espaço**: Transição instantânea da retícula para ícone de câmera para captura de janela isolada sob o cursor com realce azul Liquid Glass.
    - **Janela Isolada com Fundo Transparente e Dupla Sombra Gaússiana (`isolated_window.rs`)**:
      - Composição de canal alfa com cantos arredondados Squircle $G^2$.
      - Sombra analítica de dupla camada da Apple (Oclusão Ambiente $\sigma=4.0\text{px}$ + Penumbra Profunda $\sigma=38.0\text{px}$) com cálculo de margens de respiro (84px nas laterais, 48px no topo e 120px na base) para caimento perfeito da curva gaussiana.
      - Supressão dinâmica de sombra ao manter a tecla `Option` pressionada, gerando imagem rente à borda da janela.
  - **Barra de Ferramentas HUD Interativa (`src/hud/`)**:
    - `Cmd+Shift+5`: Pílula flutuante Liquid Glass com 5 modos operacionais (Capturar Tela, Capturar Janela, Capturar Parte, Gravar Tela, Gravar Parte).
    - Menu Popover de Opções: Destinos de salvamento (Mesa, Documentos, Área de Transferência, Mail, Mensagens, QuickTime Player ou pasta personalizada), Temporizador de 5s ou 10s com contagem regressiva, ativação de microfone e efeito de clique do mouse.
  - **Miniatura Flutuante com Física de Molas (`src/thumbnail/`)**:
    - Miniatura de $180\times 112\text{px}$ deslizando no canto inferior direito da tela via `qz_animation::SpringAnimation` (preset `dock_bounce`).
    - Temporizador de auto-salvamento de 5 segundos pausado automaticamente ao passar o cursor.
    - Descarte com gesto de deslizar para a direita (*swipe right*) com salvamento imediato e suporte a arrastar-e-soltar (*drag & drop*) direto para outros aplicativos.
  - **Daemon Gravador de Tela PipeWire (`src/recorder/`)**:
    - Pipeline de captura de vídeo com aceleração por hardware via DMA-BUF (codecs HEVC/H.264 a 60fps/120fps ProMotion).
    - Loopback de áudio do sistema com captura opcional de microfone.
    - Máquina de estados: Inativo, Contagem Regressiva, Gravando, Pausado e Finalizando.
    - Cápsula indicadora roxa na Top Menu Bar com contador de tempo decorrido e atalho `Cmd+Ctrl+Esc` para encerramento rápido.
  - **Controlador Mestre de Capturas (`src/controller.rs`)**:
    - Orquestração unificada de atalhos globais, retículas, HUD, geração de arquivos nomeados por carimbo de data/hora e entrega à área de transferência ou disco.

---

## [0.7.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-quicklook` (Fase 7 - Daemon de Pré-Visualização com Barra de Espaço, Expansão 120fps, Marcação, Recorte de Mídia e Visual Look Up)**:
  - **Daemon de Barra de Espaço & Retargeting em Tempo Real (`src/controller.rs`)**:
    - Disparo instantâneo por tecla `Espaço` com fechamento automático em `Espaço` ou `Esc`.
    - Troca fluida em pleno voo de arquivos sem reabrir a janela ao navegar pelas teclas direcionais no Finder.
  - **Motor de Expansão e Colapso Geométrico a 120fps (`src/animation/`)**:
    - Interpolação analítica $C^2$ contínua (`rect_expansion.rs`) entre o retângulo da miniatura de origem $[x, y, w, h]$ e o retângulo da janela de pré-visualização centralizada $[x, y, w, h]$ via física de molas (`qz_animation::SpringAnimation`).
    - Tratamento de interrupção com reversão elástica sem descontinuidades ou saltos bruscos.
  - **Geradores de Pré-Visualização Multi-Formato (`src/preview/`)**:
    - **Imagens**: Suporte a Display P3, sRGB, transparência alfa e formatos raster/vetoriais (HEIC, PNG, JPEG, WebP, SVG, TIFF).
    - **Documentos PDF**: Contador de páginas, índice estruturado (TOC) e metadados de título.
    - **Código-Fonte e Texto**: Destaque e contagem de linhas para Rust, Python, C++, TypeScript, TOML, JSON e Markdown.
    - **Mídia (Áudio e Vídeo)**: Resolução, taxa de quadros (60fps/120fps), codecs de vídeo (HEVC, ProRes, AV1) e áudio (AAC, FLAC, ALAC).
    - **Arquivos Compactados e Pastas**: Inspeção de diretórios ZIP/DMG com lista de entradas e cálculo de tamanho descompactado.
  - **Subsistema de Marcação e Anotações Interativas (`src/markup/`)**:
    - Ferramentas de Caneta com variação de pressão, Marca-Texto com blend multiplicativo, Flechas vetoriais, Formas geométricas (Retângulo, Elipse), Notas de texto e Assinatura por trackpad.
    - Suavização de traços por splines cúbicas / Catmull-Rom e histórico local de Desfazer/Refazer.
    - Rotação horária de 90° (`Cmd+R`).
  - **Subsistema de Recorte Não-Destrutivo de Mídia (`src/trim/`)**:
    - Marcadores táteis de Ponto de Entrada (*In-Point*) e Ponto de Saída (*Out-Point*) com scrubber sincronizado e cálculo de limites para exportação.
  - **Visual Look Up & Destaque de Assunto (*Subject Lift*) (`src/visual_lookup/`)**:
    - Segmentação automática do assunto em primeiro plano com contorno poligonal e animação cintilante dourada (*shimmer*).
    - Ação "Copiar Assunto" gerando arquivo PNG transparente isolado para a área de transferência.
    - Classificação de entidades por inteligência visual (Animais, Plantas, Monumentos, Obras de Arte, Comida, Texto) com ficha informativa popover.

---

## [0.6.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-finder` (Fase 6 - Desktop Stacks, 4 Modos de Visualização, FS Undo/Redo, Pastas com Mola, Flocking, Renomeação em Lote e Manipulação de .app/.dmg)**:
  - **Desktop Manager & Stacks Dinâmicas (`src/desktop/`)**:
    - Agrupamento inteligente de ícones da Mesa em pilhas por Tipo (Screenshots, Imagens, Documentos, Filmes, Música, Arquivos, Código), Data de Modificação, Data de Criação e Tags do Finder.
    - Animação de desdobramento/recolhimento em leque acionada por física analítica de molas (`qz_animation::SpringAnimation`).
  - **As 4 Visualizações Canônicas do Finder (`src/views/`)**:
    - **Visualização em Ícones (`Cmd+1` / `icon_view.rs`)**: Layout livre ou em grade magnética, escala de ícones de 16px a 512px, rótulos na parte inferior ou à direita, metadados sob o rótulo e cálculo de alinhamento por célula.
    - **Visualização em Lista (`Cmd+2` / `list_view.rs`)**: Árvore hierárquica com triângulos de abertura, colunas redimensionáveis e ordenáveis (Nome, Data de Modificação, Tamanho, Tipo), e opção de cálculo em tempo real de tamanho de pastas.
    - **Visualização em Colunas (`Cmd+3` / `column_view.rs` - NeXT Miller Columns)**: Cadeia horizontal infinita de colunas com navegação completa por setas direcionais (`Esquerda`/`Direita`/`Cima`/`Baixo`) e coluna direita de pré-visualização com metadados e botões de Ações Rápidas (*Quick Actions*).
    - **Visualização em Galeria (`Cmd+4` / `gallery_view.rs` - Evolução do Cover Flow)**: Carrossel central de destaque visual de alta resolução, faixa inferior de miniaturas horizontais e painel lateral colapsável de Inspeção com atributos EXIF.
  - **Mecanismo Transacional de Undo/Redo do Sistema de Arquivos (`src/undo/`)**:
    - Diário atômico para `Cmd+Z` e `Shift+Cmd+Z` com reversão segura de operações de Mover, Copiar, Renomear, Enviar para a Lixeira, Restaurar da Lixeira (*Put Back*), Criar Pasta e Renomeação em Lote.
  - **Pastas com Mola e Física de Arraste em Bando (`src/drag_physics/`)**:
    - *Spring-Loaded Folders*: Abertura elástica automática de pastas ao pairar itens arrastados por 500ms (ou ao pressionar a barra de `Espaço`).
    - *Multi-Item Flocking*: Convergência orgânica de múltiplos itens arrastados em direção ao cursor com amortecimento harmônico de 2ª ordem e rotação angular suave (*splay* de $\pm 4^\circ$).
  - **Motor de Renomeação em Lote do macOS Tahoe 26 (`src/batch_rename/`)**:
    - 3 modos de transformação: Substituir Texto (com ou sem diferenciação de maiúsculas/minúsculas), Adicionar Texto (prefixo ou sufixo preservando extensões) e Formato Sequencial (Nome e Índice, Contador `0001` ou Data).
    - Geração de pré-visualização em tempo real e detecção de colisões de nomes.
  - **Manipulador de Bundles `.app` e Imagens de Disco `.dmg` (`src/bundle_dmg/`)**:
    - Resolução estrutural de pacotes de aplicação (`Contents/Info.plist`, binário principal em `Contents/MacOS/` e recursos de ícone).
    - Montagem em loopback de imagens `.dmg` e `.iso` no diretório virtual `/Volumes/` com ciclo de vida e ação de ejetar.
  - **Controlador Mestre de Janelas do Finder (`src/window.rs`)**:
    - Barra de ferramentas Liquid Glass, barra lateral com seções de Favoritos, Locais e Discos montados, barra de caminho de diretório e barra de status com capacidade restante de armazenamento.

---

## [0.5.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-spotlight` (Fase 5 - Spotlight Launcher de 4 Modos do macOS Tahoe 26)**:
  - **Modo 1: Busca Universal & Motores de Avaliação em Tempo Real (`src/search/`)**:
    - **Calculadora Científica com Pratt Parser (`calc.rs`)**: Avaliador sintático de precedência por binding power suportando exponenciação associativa à direita (`^`), funções trigonométricas/hiperbólicas/logarítmicas (`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `sqrt`, `cbrt`, `log`, `ln`, `abs`, `exp`), notação científica e constantes matemáticas canônicas ($\pi$, $e$, $\tau$, $\phi$).
    - **Conversor de Unidades & Moedas em Linguagem Natural (`converter.rs`)**: Conversão direta entre unidades de Comprimento (m, km, cm, mm, in, ft, yd, mi), Massa (kg, g, mg, lb, oz, ton), Temperatura (°C, °F, K), Armazenamento Digital decimal e binário (B, KB, MB, GB, TB, KiB, MiB, GiB, TiB), Tempo e Cotações Monetárias (USD, EUR, GBP, JPY, BRL, CAD, AUD, CHF, CNY).
    - **Indexador e Busca OCR (`ocr.rs`)**: Mapeamento de texto reconhecido em documentos escaneados, capturas de tela e PDFs com caixas delimitadoras normalizadas ($[x, y, w, h]$), pontuação de confiança e extração contextual de trechos em tempo real.
    - **Motor de Busca Universal (`engine.rs`)**: Agregação instantânea de aplicativos, preferências de sistema, documentos e contatos, priorizando cartões de cálculo e conversão com algoritmo de ranqueamento fuzzy em 4 camadas (correspondência exata, prefixo, limite de palavra e subsequência) acrescido de impulso por frequência de uso.
  - **Modo 2: Ações de Sistema e Executor de Menus de Aplicação (`src/actions/`)**:
    - **Registro de Ações do Sistema (`system_actions.rs`)**: Acionamento direto de Repouso, Reinicialização, Desligamento, Bloqueio de Sessão (`⌃⌘Q`), Encerramento de Sessão (`⇧⌘Q`), Esvaziar Lixeira (`⇧⌘⌫`), Modo Escuro/Claro, rádios Wi-Fi e Bluetooth, Economia de Energia, Night Shift, Capturas de Tela (`⇧⌘3`, `⇧⌘4`, `⇧⌘5`), Monitor de Atividade e Forçar Encerramento (`⌥⌘⎋`).
    - **Executor de Menus do App Ativo (`menu_runner.rs`)**: Travessia recursiva e achatamento da hierarquia de menus do aplicativo em primeiro plano (`File > Export As > PDF...`), permitindo pesquisa rápida e disparo de comandos diretamente pelo teclado.
  - **Modo 3: Histórico Nativo da Área de Transferência (`src/clipboard/`)**:
    - Armazenamento em ring buffer com capacidade configurável (padrão de 200 itens) para texto simples, texto formatado, imagens e arquivos.
    - **Detector Automático de Swatch de Cores**: Reconhecimento instantâneo de strings de cores nos formatos Hex (`#RGB`, `#RRGGBB`, `#RRGGBBAA`) e `rgb()` com síntese de pré-visualização de amostra de cor ao vivo.
    - **Proteção de Itens Fixados**: Sistema de marcação (*pin*) que imuniza itens favoritos contra descarte quando o buffer atinge sua capacidade máxima.
  - **Modo 4: Biblioteca de Aplicativos Categorizada (`src/app_library/`)**:
    - Grade semântica de aplicativos agrupados por categorias canônicas: Produtividade & Finanças, Utilitários, Ferramentas de Desenvolvedor, Criatividade & Design, Mídia & Entretenimento, Social & Comunicação, e Sistema & Ajustes.
    - **Modo Jiggle com Física Oscilatória**: Oscilação angular harmônica a $8.0\text{Hz}$ ($\pm 1.5^\circ$) com desfasamento angular entre itens para efeito orgânico não-sincronizado e distintivo de remoção rápida.
    - Rastreamento analítico de contagem de lançamentos e ordenação automática por popularidade.
  - **Controlador Mestre do Spotlight (`src/controller.rs`)**:
    - Transição contínua de altura via física analítica de mola (`qz_animation::SpringAnimation`) entre a pílula compacta de 56px e até 520px de altura expandida.
    - Navegação completa por teclado com atalhos de troca de modo `Cmd+1` a `Cmd+4`, setas direcionais com rotação cíclica, ativação por `Enter` e expansão de Quick Look na tecla `Espaço`.

---

## [0.4.0-alpha.1] - 2026-10-15

### Added
- **Crate `qz-shell` (Fase 4 - Shell, Top Menu Bar, Control Center, Notificações, Widgets e Tripartite Dock)**:
  - **Top Menu Bar com Suporte a Notch (`src/menubar/`)**:
    - Detecção de entalhe físico (*Notch*) e particionamento de menus (`notch.rs`), realizando o salto contínuo de itens de menu da esquerda para a direita do entalhe sem clipping.
    - Menu Canônico da Maçã (`apple_menu.rs`) com submenu de Itens Recentes, diálogo de Forçar Encerramento (`Opt+Cmd+Esc`) e mutação dinâmica instantânea com tecla `Option` (execução direta de Repouso/Reiniciar/Desligar sem o diálogo de confirmação de 60 segundos).
    - Menu Global com fallback automático para o Finder caso o app ativo não forneça menu (`global_menu.rs`), hover lateral instantâneo (0ms de latência), duplo flash de 120ms ao clicar, itens dinâmicos com `Option`, Lista de Janelas ativas com marcações `✓` (em foco) e `♦` (minimizada), e Help Search Rector com busca fuzzy de atalhos e acionamento direto via teclado.
    - Bandeja de Ícones de Status (`status_items.rs`) com suporte a remoção por arraste `Cmd+Drag` com efeito "Poof" ao exceder 45px de distância, e menus de diagnóstico ao clicar com `Option` (detalhes de Wi-Fi BSSID/RSSI, áudio e Bluetooth).
    - Cápsula de Privacidade Animada (`privacy_capsule.rs`) em Liquid Glass com indicadores de hardware para microfone (Laranja `#FF9500`), câmera (Verde `#34C759`) e gravação de tela (Roxo `#AF52DE`), expandindo com mola `CASpringAnimation` para exibir apps ativos no clique.
  - **Control Center Modular (`src/control_center/`)**:
    - Pílula flutuante Liquid Glass com módulos independentes: Wi-Fi, Bluetooth, AirDrop, Modos de Foco (Não Perturbe, Trabalho, Pessoal, Sono com duração personalizável), Stage Manager e Espelhamento de Tela.
    - Sliders táteis de Brilho da Tela e Volume de Áudio com suporte a True Tone, Night Shift, Dark Mode e Economia de Energia.
    - Módulo "Tocando Agora" (*Now Playing*) com quantizador de cores **Median Cut** em CPU para extração em tempo real da paleta dominante da capa do álbum.
  - **Central de Notificações com Pilhas 3D (`src/notifications/`)**:
    - Agrupamento inteligente de notificações por aplicativo em pilhas 3D táteis com escala e elevação decrescente.
    - Expansão de pilha fluida acionada por clique ou gesto, botões de ação customizados e resposta rápida inline (*inline reply*).
  - **Sistema de Widgets do macOS Tahoe 26 (`src/widgets/`)**:
    - Suporte a widgets interativos na Mesa e na barra lateral nos formatos Pequeno ($2\times 2$), Médio ($4\times 2$) e Grande ($4\times 4$).
    - Transição Cromática Adaptativa de 250ms (*Adaptive Chromatic Transition*): ao focar em uma janela sobreposta ou trocar de espaço, os widgets desvanecem suavemente suas cores vibrantes para tons monocromáticos de vidro Liquid Glass.
  - **Tripartite Dock com Física e Modos Avançados de Stacks (`src/dock/`)**:
    - Layout em 3 seções canônicas: Apps Fixados, Apps Recentes / Em Execução (com indicador sutil de pontinho iluminado), e Pastas / Stacks / Lixeira / Janelas Minimizadas.
    - Separador vertical redimensionável com clique e arraste, com atração magnética (*snap*) para os tamanhos canônicos da Apple (48px, 64px, 96px, 128px) ao manter pressionada a tecla `Option`.
    - Ampliação contínua por cosseno parabólico e animação de salto (*dock bounce*) para notificações e abertura de apps.
    - Efeito Spring-Loaded Exposé ao arrastar arquivos sobre o ícone de uma aplicação no Dock por 700ms.
    - Visualizador de Stacks completo com 4 modos de exibição: Leque (*Fan* com curvatura parabólica), Grade (*Grid* com rolagem fluida), Lista (*List*) e Automático (*Auto* determinando o modo pelo número de itens).
    - Redução temporal para $0.15\times$ (Slow-Motion) de todas as animações de abertura de pilhas ao segurar `Shift`.
    - Mutação dinâmica do ícone de Lixeira para Ejetar (*Eject*) com highlight vermelho sutil quando um volume montado ou drive DMG é arrastado em sua direção.

---

## [0.3.0-alpha.1] - 2026-10-15

### Added
- **Subsistemas de Core Compositor em `qz-compositor` (Fase 3)**:
  - **ProMotion Dynamic VRR Frame Pacing Engine (`src/vrr.rs`)**:
    - Agendador dinâmico de frame budget com suporte de 10Hz a 120Hz (120Hz para molas/gestos/cursor recente em 100ms, 60Hz para vídeo/UI, 10Hz–24Hz para repouso).
    - Controle da propriedade DRM `VRR_ENABLED` em conectores compatíveis com `vrr_capable == 1`.
  - **Motor de Escala Retina e DPI Físico (`src/retina.rs`)**:
    - Leitura de dimensões físicas e cálculo de densidade DPI precisa a partir do EDID de monitores DRM.
    - Quantizador fracionário em múltiplos exatos de $1/120$ para conformidade com `wp-fractional-scale-v1`.
    - Ponte de compatibilidade para XWayland legado renderizado em `@2x` inteiro com matriz de downsampling bicúbico Catmull-Rom.
  - **Motor de Gestos Multi-Touch 1:1 (`src/gestures.rs`)**:
    - Natural Scrolling com física hiperbólica Cocoa Rubber-Banding nos limites do viewport.
    - Pinça com 2 dedos (Zoom contínuo) e duplo toque de 2 dedos (Smart Zoom).
    - Toque de 3 dedos para Dicionário e Detectores de Dados.
    - Swipe de 2 dedos para navegação em histórico com sobreposição fluida de páginas.
    - Swipe de 3 ou 4 dedos: Cima (Mission Control), Baixo (App Exposé), Lados (troca contínua 1:1 de Spaces).
    - Pinch Out com 4 dedos para Mostrar Mesa (*Show Desktop*) e Pinch In para App Library / Launchpad.
    - `Ctrl` + Scroll de 2 dedos para ampliação suave de acessibilidade da tela inteira.
  - **Gerenciador de Spaces, Stage Manager, Hot Corners e PiP (`src/spaces.rs`)**:
    - Gestão de desktops virtuais Spaces com Split View nativo (divisão 50/50 com controle de proporção de 0.25 a 0.75).
    - Stage Manager com app ativo central e até 4 grupos recentes na lateral esquerda com inclinação 3D (`rotateY(14°)`).
    - Picture-in-Picture (PiP) magnético atraído por molas para os 4 cantos da tela (16px padding), movimento livre ao segurar `Cmd` e recolhimento lateral na borda da tela.
    - Avaliador de Hot Corners (Cantos Ativos) para os 4 cantos da tela com acionamento padrão de Nota Rápida (*Quick Note*).

---

## [0.2.0-alpha.1] - 2026-10-15

### Added
- **Crate Gráfica `qz-renderer`**:
  - Implementação completa da infraestrutura de renderização híbrida Vulkan 1.3 e OpenGL ES 3.2.
  - Módulo de importação de memória Zero-Copy DMA-BUF (`crates/qz-renderer/src/dmabuf.rs`) via extensões `VK_EXT_external_memory_dma_buf` e `VK_EXT_image_drm_format_modifier`.
  - Mapeamento direto de formatos DRM FourCC Linux (`DRM_FORMAT_ARGB8888`, `DRM_FORMAT_XRGB8888`, `DRM_FORMAT_ABGR8888`, `DRM_FORMAT_XBGR8888`) para `vk::Format` sem cópia de buffer CPU-GPU.
- **Coleção Completa de Shaders GLSL 450 Compiláveis**:
  - `kawase_down.frag` e `kawase_up.frag`: Pirâmide Dual-Kawase Blur de 8 passes divididos em 4 passes de downsampling e 4 passes de upsampling com preservação de largura de banda de memória em resoluções 1/2, 1/4, 1/8 e 1/16.
  - `liquid_glass_tahoe.frag`: Shader de lente refrativa Liquid Glass do macOS Tahoe 26 com SDF superelipse $G^2$ (Squircle com norma $L^4$), dispersão cromática radial e de borda, saturação luma Rec.709 ($S=1.85$), blending dinâmico de vivacidade (Color Burn no Dark Mode / Color Dodge no Light Mode), rim highlight especular de 1px sensível à direção da luz e dithering procedural anti-banding via ruído *Interleaved Gradient Noise*.
  - `genie.vert` e `genie.frag`: Shaders da malha de tesselação $64 \times 64$ ($4225$ vértices e $24576$ índices) com deformação não-linear baseada em curvas suaves de Hermite para minimizar janelas no Dock com Efeito Gênio.
  - `shadow_sdf.frag`: Sombra analítica de dupla camada da Apple (Ambient Occlusion $\sigma=4.0\text{px}$ + Penumbra Profunda $\sigma=38.0\text{px}$) resolvida por integração gaussiana contínua via aproximação de função erro ($\text{erf}$), além de perfil dedicado para *Floating Glass Pills* ($\sigma=10.0\text{px}$).
  - `catmull_rom.frag`: Filtro bicúbico de reconstrução Catmull-Rom de 9 amostras para escala Retina a partir de buffers legados renderizados em `@2x` inteiro.
  - `cube_3d.vert` e `cube_3d.frag`: Projeção de perspectiva e matriz MVP para a rotação de cubo 3D da Troca Rápida de Usuário (*Fast User Switching*).
- **Pipelines Rust em `crates/qz-renderer/src/pipeline/`**:
  - `kawase.rs`: Gerenciador da cadeia de pirâmide e cálculo de push constants `half_pixel` e `offset`.
  - `liquid_glass.rs`: Construtor de uniforms de vidro para janelas normais e pílulas flutuantes, incluindo avaliador de SDF squircle em CPU.
  - `genie.rs`: Gerador geométrico estático da malha de grade $64 \times 64$.
  - `shadow.rs`: Calculador de extensões de margem de sombra analítica para tesouramento de janelas.
  - `catmull_rom.rs`: Calculador de fatores de escala e resolução física para display Retina.
  - `cube_3d.rs`: Solver de matriz MVP com câmera de $45^\circ$ FOV e rotação angular suave acionada por física de mola.
- **Integração Contínua (CI) em Nuvem com GitHub Actions**:
  - Arquivo `.github/workflows/ci.yml` configurado com runners Ubuntu para compilação, testes e validação automática de todos os alvos e sintaxe de protocolos XML a cada `push`.
  - Autenticação e vinculação ao repositório oficial [nepotira/quartz-desktop](https://github.com/nepotira/quartz-desktop).

---

## [0.1.0-alpha.1] - 2026-10-15

### Added
- **Arquitetura do Workspace Root**:
  - `Cargo.toml` raiz do workspace configurado para Rust Edition 2024, resolução 3, dependências de alto nível (`smithay`, `wayland-server`, `ash`, `glow`, `glam`, `thiserror`, `anyhow`, etc.) e perfis de release otimizados (`lto = "thin"`, `codegen-units = 1`, `opt-level = 3`).
- **Protocolos Wayland Customizados (XML)**:
  - `protocol/quartz-shell-unstable-v1.xml`: Protocolo para Liquid Glass, Floating Glass Pills (margens de 8px), semáforos (Traffic Lights), estado dirty de documento não salvo, proxy icons arrastáveis, sheets modais e abas universais destacáveis.
  - `protocol/quartz-global-menu-v1.xml`: Protocolo para Global Menu Bar, mutação dinâmica de rótulos com a tecla `Option`, sincronização do rodapé do menu Janela (`✓` ativa, `♦` minimizada), Help Search Rector com vetor de posição para seta azul flutuante e flash de seleção de item.
  - `protocol/quartz-workspace-ipc-v1.xml`: Protocolo de IPC privilegiado para captura de tela com canal alfa e sombra real de dupla camada, Quick Look com expansão retangular a 120fps a partir do ícone, PiP magnético e Desktop Reveal.
- **Crate de Física de Animação `qz-animation`**:
  - Solver analítico fechado de equações diferenciais ordinárias (EDO) de 2ª ordem para oscilador harmônico amortecido em tempo contínuo $C^2$ (regimes sub-amortecido, criticamente amortecido e super-amortecido).
  - Suporte a redirecionamento contínuo em pleno voo (`retarget`) sem sobressaltos ou descontinuidades na velocidade.
  - Presets físicos oficiais: `mission_control()`, `window_open()`, `dock_bounce()`, `stage_manager()`, `rubber_return()` e `cursor_shake()`.
  - Abstrações multidimensionais `Spring2D` e `SpringRect`.
  - Equação hiperbólica de elasticidade Cocoa (Rubber-Banding) com transformações direta e inversa.
  - Detector vetorial "Shake to Find Cursor" com janela deslizante de 400ms, limiar de velocidade $>1200\text{px/s}$, inversão angular $>120^\circ$ e mola de expansão de $1.0\times$ para $8.0\times$.
  - Magnificação parabólica por cosseno elevado e deslocamento conservativo de ícones no Dock.
- **Entrada Inteligente e Click-Through em `qz-compositor`**:
  - Smart Keymapper (`crates/qz-compositor/src/input/keymapper.rs`) com tradução de `Cmd` $\leftrightarrow$ `Ctrl` em aplicativos gráficos e preservação estrita de sinais POSIX (`Ctrl+C` = SIGINT, `Ctrl+Z` = SIGTSTP, `Ctrl+D` = EOF) em emuladores de terminal.
  - Navegação universal de texto Cocoa/Emacs (`Ctrl+A/E/D/B/F/N/P/Y`, `Opt+Setas`, `Cmd+Setas`, `Opt+Backspace`, `Cmd+Backspace`).
  - Ajuste fino de volume e brilho em $1/4$ de passo (64 níveis discretos) via `Shift + Option`.
  - Despachador de Click-Through (`crates/qz-compositor/src/input/click_through.rs`) permitindo operar controles e semáforos de janelas em segundo plano segurando `Cmd` sem alterar a profundidade Z-order e sem transferir o foco de teclado.
  - Roteamento de scroll desfocado diretamente para a superfície sob o cursor do mouse.
  - Temporizador de 500ms no botão verde para ativação do Window Tiling Popover.

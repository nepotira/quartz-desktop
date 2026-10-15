# Registro de Alterações (Changelog)

Todas as alterações notáveis neste projeto serão documentadas neste arquivo.

O formato baseia-se no [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Semantic Versioning (SemVer 2.0.0)](https://semver.org/lang/pt-BR/).

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

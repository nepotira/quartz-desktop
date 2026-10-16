# Diretrizes de Engenharia de Software, Documentação, Controle de Versão Git/GitHub e Versionamento Semântico
## Projeto: QuartzLinux (`qz-desktop`) — Clean Slate macOS Tahoe 26 Architecture

---

## 1. Visão Geral e Mandato de Governança

Este documento institui o **Protocolo Oficial de Engenharia de Software** que deve ser cumprido de maneira incondicional a cada alteração realizada no repositório **QuartzLinux (`qz-desktop`)**, por menor ou mais pontual que seja.

Toda intervenção no código-fonte deve obedecer simultaneamente aos **3 Pilares de Governança de Engenharia**:
1. **Documentação Técnica, Matemática e Arquitetural Viva (`#![deny(missing_docs)]` + `CHANGELOG.md` + Especificação de Shaders GLSL 450)**
2. **Controle de Versão Profissional Git & GitHub (Isolamento em Branches, Conventional Commits 1.0.0, Merges Rastreáveis e Compilação 100% em Nuvem via GitHub CLI `gh`)**
3. **Versionamento Semântico Estrito (SemVer 2.0.0 Sincronizado entre `Cargo.toml`, `CHANGELOG.md` e Git Annotated Tags)**

---

## 2. As 6 Leis Anti-Fraude Gráfica (Zero-Tolerance Policy)

Nenhuma alteração pode ser comitada ou mesclada se violar qualquer uma das 6 leis abaixo:

1. **PROIBIDO USAR FONTES, GLIFOS OU UNICODE PARA ÍCONES (ZERO `▯` TOFU):**
   - É terminantemente proibido usar caracteres Unicode privados (`\u{F8FF}`), emojis ou glifos de fontes de texto para desenhar o logotipo da Maçã, ícones da barra superior (Wi-Fi, Bateria, Busca, Control Center), ícones da Sidebar do Finder, pastas ou ícones do Dock.
   - O renderizador de fontes (`FreeType`/`HarfBuzz`) só pode receber texto alfanumérico legível. **100% dos ícones do sistema devem ser renderizados matematicamente via Shaders Procedurais (SDFs compostos, curvas de Bézier cúbicas e gradientes multi-stop) ou paths vetoriais rasterizados via GPU (`qz-icons`)**.
2. **PROIBIDO FINGIR LIQUID GLASS COM ALPHA BLENDING (`alpha = 0.7`):**
   - É proibido pintar a Sidebar, a Menu Bar, o Spotlight ou o Dock usando apenas uma cor semi-transparente no mesmo RenderPass do papel de parede. Nenhuma linha diagonal ou forma do papel de parede pode jamais aparecer nítida atrás do vidro.
   - O pipeline deve renderizar o fundo em `FBO_Scene`, executar a **Pirâmide Dual-Kawase de 5 Níveis de Downsample + 5 Níveis de Upsample ($\sigma_{\text{eq}} \ge 48\text{px}$)** e/ou o *Vogel Disk Blur* em FBO separado, aplicar refração ótica de lente nas bordas curvas ($G^2$ Squircle), aberração cromática, saturação Luma $1.95\times$, *Color Dodge/Burn*, *Interleaved Gradient Noise* e borda especular de `1.0px` + hairline externa de `0.5px`.
3. **PROIBIDO SOMBRAS RETANGULARES DURAS:**
   - Jamais desenhe um retângulo preto/cinza deslocado atrás de uma janela. Toda sombra deve ser calculada analiticamente no Fragment Shader em um quad expandido (`+90px` em cada direção) usando a aproximação polinomial de Abramowitz & Stegun para a Função de Erro Gaussiana (`erf`) em **duas camadas simultâneas** (Ambient Occlusion de contato $\sigma_1 = 4\text{px}, \Delta y_1 = 2\text{px}$ + Penumbra profunda $\sigma_2 = 42\text{px}, \Delta y_2 = 24\text{px}$).
4. **PROIBIDO ARCOS CIRCULARES COMUNS:**
   - Todos os cantos arredondados de janelas, Sidebars, Dock, menus, popovers e ícones devem usar a Função de Distância com Sinal (SDF) de **Superelipse / Squircle com continuidade de curvatura $G^2$** (norma $L_p$ com $p = 2.35$ nos cantos), nunca círculos simples.
5. **PROIBIDO COLAR A SIDEBAR NA BORDA OU USAR BORDAS BRANCAS DE HTML:**
   - No **macOS Tahoe 26**, a Sidebar do Finder e dos Ajustes é uma **Floating Glass Pill** descolada **`10px`** da esquerda, do topo e da base da janela (`x = win.x + 10`, `y = win.y + 10`, `w = 205`, `h = win.h - 20`, `R = 11px`), flutuando com sombra própria e sem linha reta vertical dividindo a janela. Os botões da Toolbar (`<`, `>`, Visualizações, Busca) **não possuem bordas brancas duras de 1px**.
6. **PROIBIDO SELEÇÃO ESTILO WINDOWS EXPLORER NO FINDER:**
   - Na visualização em Ícones (`Cmd+1`) do Finder, quando uma pasta ou arquivo está selecionado, é proibido desenhar um retângulo azul englobando o ícone e o texto juntos:
     - Atrás do **ícone** apenas: um quadrado arredondado neutro (`rgba(255, 255, 255, 0.12)`, `R = 10px`) **sem nenhuma borda azul**.
     - Atrás do **rótulo de texto** apenas: uma pílula azul sólida (`#0058D0`, `R = 4.5px`, padding `2px 6px`) com texto branco `#FFFFFF` em Semibold.

---

## 3. Pilar I — Regras de Documentação Obrigatória (Documentation Integrity)

### A. Política `#![deny(missing_docs)]` em Todo o Workspace
- Toda crate do workspace (`qz-animation`, `qz-icons`, `qz-renderer`, `qz-compositor`, `qz-shell`, `qz-spotlight`, `qz-finder`, `qz-quicklook`, `qz-screenshot`, `qz-daemons`, `qz-session-lock`, `qz-toolkit`, apps e compat) deve declarar `#![deny(missing_docs)]` em seu arquivo raiz (`src/lib.rs`).
- **Nenhum item público pode existir sem documentação Rustdoc completa:**
  - Módulos (`//! # Nome do Módulo`) explicando propósito arquitetural, papel no macOS Tahoe 26 e invariantes matemáticos.
  - Estruturas (`struct`), campos públicos (`pub field: Type`), enumerações (`enum`), variantes e métodos (`pub fn`).
  - Constantes (`pub const`), tipos associados (`type`) e traits (`pub trait`).

### B. Documentação Científica e Matemática
- Toda equação diferencial (molas $C^2$ contínuas), integral Gaussiana (`erf`), SDF de Superelipse $G^2$ ou transformação de cor Rec.709 deve ser documentada em notação matemática LaTeX/KaTeX (`$...$` ou `$$...$$`) diretamente nas docstrings do item correspondente.
- Proibição absoluta de `// TODO`, `todo!()`, `unimplemented!()` ou placeholders.

### C. Manutenção Viva do `CHANGELOG.md`
- **Cada alteração realizada no repositório — por mínima que seja — deve obrigatoriamente ser registrada no arquivo [`CHANGELOG.md`](file:///home/nepo/Downloads/MacTux/CHANGELOG.md)** na raiz do projeto, seguindo rigorosamente o padrão internacional [Keep a Changelog 1.1.0](https://keepachangelog.com/pt-BR/1.1.0/):
  - `Added`: Novos crates, shaders, módulos, animações ou recursos.
  - `Changed`: Evoluções arquiteturais, refinamentos matemáticos ou ajustes de parâmetros.
  - `Fixed`: Correções de bugs, ajustes de geometria ou conformidade com as 6 Leis Anti-Fraude.
  - `Removed`: Remoção de código obsoleto.
  - `Security`: Garantias de segurança de memória e eliminação de pânicos.

---

## 4. Pilar II — Controle de Versão Git, Branches e Compilação em Nuvem via GitHub CLI (`gh`)

### A. Regra Absoluta de Compilação Remota (Zero Local Build)
- **É TERMINANTEMENTE PROIBIDO executar `cargo build`, `cargo check`, `cargo test` ou `cargo clippy` na máquina local do usuário.**
- Toda compilação, verificação de tipos e execução da suíte de testes do workspace deve ocorrer **exclusivamente nos runners remotos do GitHub Actions**, acionados e monitorados via GitHub CLI (`gh`):
  1. Enviar o branch para o repositório remoto: `git push -u origin <branch>`
  2. Acionar ou inspecionar o workflow de CI: `gh run list --branch <branch>` / `gh workflow run ci.yml --ref <branch>`
  3. Acompanhar a compilação em nuvem até concluir: `gh run watch <run-id> --exit-status` ou `gh run view <run-id> --log-failed`
  4. Somente após o status `✓ SUCCESS` no GitHub Actions o branch pode ser mesclado à `main` e receber sua tag SemVer.

### B. Estratégia de Branches Semânticos
Nenhuma alteração deve ser desenvolvida diretamente na `main`. Toda tarefa exige a criação de um branch dedicado:

| Prefixo de Branch | Finalidade | Exemplo |
|---|---|---|
| `main` | Ramo principal estável. Sempre compilado e validado pelo GitHub Actions. | `main` |
| `feat/*` | Implementação de novas fases, crates (`qz-icons`), shaders ou subsistemas. | `feat/phase-1-2-clean-slate-core` |
| `fix/*` | Correção cirúrgica de bugs matemáticos, visuais ou de protocolo. | `fix/finder-bipartite-selection-pill` |
| `docs/*` | Atualização de normas de engenharia, arquitetura, README e CHANGELOG. | `docs/engineering-governance-v2` |
| `refactor/*` | Reestruturação interna mantendo invariantes e cobertura de testes. | `refactor/kawase-5-level-pyramid` |
| `test/*` | Adição ou expansão de testes unitários e de integração. | `test/animation-18-catalog-suite` |
| `ci/*` | Ajustes no pipeline `.github/workflows/ci.yml`. | `ci/enable-all-feature-branches` |

### C. Commits Convencionais (Conventional Commits 1.0.0)
Todo commit deve ser atômico, claro e formatado segundo o padrão Conventional Commits:

```text
<tipo>(<escopo>): <descrição concisa no imperativo>

- Detalhamento técnico 1 (equações, arquivos e subsistemas afetados)
- Detalhamento técnico 2 (conformidade com as 6 Leis Anti-Fraude)

Refs: #<issue/phase> | Version: vX.Y.Z-alpha.N
```

- **Tipos permitidos:** `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `ci`, `chore`.
- **Escopos oficiais:** `standards`, `workspace`, `animation`, `icons`, `renderer`, `compositor`, `shell`, `spotlight`, `finder`, `quicklook`, `screenshot`, `daemons`, `session-lock`, `toolkit`, `apps`, `compat`, `ci`.

---

## 5. Pilar III — Versionamento Semântico Estrito (SemVer 2.0.0)

O repositório adota estritamente o [Semantic Versioning 2.0.0](https://semver.org/lang/pt-BR/):

$$\text{VERSÃO} = \text{MAJOR}.\text{MINOR}.\text{PATCH}-\text{PRERELEASE}$$

### A. Critérios de Incremento de Versão
1. **MAJOR (`X.0.0`):** Mudanças arquiteturais incompatíveis na API pública do workspace ou lançamento de produção `1.0.0`.
2. **MINOR (`0.Y.0-alpha.N`):** Conclusão de uma Fase Arquitetural, adição de uma nova crate (`qz-icons`), novo pipeline de shaders ou novo subsistema completo do macOS Tahoe 26.
3. **PATCH (`0.Y.Z-alpha.N`):** Correções de bugs, refinamentos visuais/matemáticos, melhorias de documentação ou ajustes de CI sem quebra de compatibilidade.

### B. Tripla Sincronização Obrigatória de Versão
A cada nova versão lançada, **os 3 artefatos abaixo devem estar rigorosamente sincronizados**:
1. **`Cargo.toml` (Raiz do Workspace):** Campo `[workspace.package] version = "X.Y.Z-alpha.N"` atualizado.
2. **`CHANGELOG.md`:** Nova seção `## [X.Y.Z-alpha.N] - YYYY-MM-DD` documentando exaustivamente tudo o que foi adicionado, alterado ou corrigido.
3. **Git Annotated Tag no GitHub:** Tag anotada criada com `git tag -a vX.Y.Z-alpha.N -m "Release vX.Y.Z-alpha.N: <resumo>"` e publicada no GitHub via `git push origin vX.Y.Z-alpha.N`.

---

## 6. Checklist Operacional de 10 Passos (Obrigatório a Cada Alteração)

Para **toda e qualquer alteração**, siga invariavelmente esta sequência:

1. **Branch Isolado:** Criar branch semântico a partir da `main` (`git checkout -b <tipo>/<nome>`).
2. **Conformidade Anti-Fraude:** Auditar o código contra as 6 Leis Anti-Fraude Gráfica (Zero Font Icons, 5-Level Kawase + Refração Liquid Glass, Sombra Dupla `erf`, Squircle $G^2$ $p=2.35$, Floating Pill Sidebar `10px` inset sem bordas brancas duras na Toolbar, Seleção Bipartida no Finder).
3. **Zero Placeholders:** Confirmar ausência total de `todo!()`, `unimplemented!()` ou `// TODO`.
4. **Documentação Rustdoc & Shaders:** Garantir que todos os símbolos públicos obedeçam a `#![deny(missing_docs)]`.
5. **Testes Unitários:** Escrever ou atualizar testes unitários cobrindo as novas funções e invariantes matemáticos.
6. **Sincronização SemVer no `Cargo.toml`:** Incrementar a versão semântica do workspace de acordo com o escopo da mudança.
7. **Registro no `CHANGELOG.md`:** Documentar a versão, data e todas as alterações nas categorias do Keep a Changelog.
8. **Commit Convencional:** Realizar `git add` e `git commit` seguindo a especificação Conventional Commits 1.0.0.
9. **Compilação e Testes em Nuvem via GitHub CLI (`gh`):** Fazer `git push` e validar a compilação/testes remotamente no GitHub Actions (`gh workflow run` / `gh run watch`) — **jamais compilar na máquina local**.
10. **Merge Profissional e Tag Anotada:** Após `✓ SUCCESS` na nuvem, mesclar na `main` (`git merge --no-ff`), criar a tag anotada `vX.Y.Z-alpha.N` e publicar (`git push origin main --tags`).

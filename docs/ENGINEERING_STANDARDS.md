# Diretrizes de Engenharia de Software, Versionamento e Documentação
## Projeto: QuartzLinux (`qz-desktop`) — macOS Tahoe 26 Architecture

---

## 1. Visão Geral e Princípios Fundamentais

Este documento estabelece as **Regras Mandatórias de Engenharia de Software** que devem ser seguidas de forma estrita e sem exceções para toda e qualquer alteração no repositório **QuartzLinux (`qz-desktop`)**, por mais microscópica ou pontual que seja.

O objetivo deste projeto é construir um Compositor Wayland e Ambiente Desktop em Rust (Edition 2024) com fidelidade matemática, física, visual e comportamental absoluta ao **macOS Tahoe 26**. Para garantir qualidade de nível militar e estabilidade de produção, a disciplina de engenharia de software é intransigente.

---

## 2. Regras de Documentação Obrigatória (Documentation Integrity)

### A. Política `missing_docs` Zero-Tolerance
- Toda crate do workspace deve declarar `#![deny(missing_docs)]` em seu arquivo raiz (`src/lib.rs` ou `src/main.rs`).
- **Nenhum item público pode existir sem documentação:**
  - Módulos (`//! # Nome do Módulo`) com explicação arquitetural do propósito, papel no subsistema do macOS e relações de dependência.
  - Estruturas (`struct`), uniões e tuplas.
  - Todos os campos individuais de estruturas (`pub field: Type`).
  - Enumerações (`enum`) e todas as suas variantes.
  - Todos os campos estruturados de variantes de enum (`Variant { field: Type }`).
  - Funções livres e métodos associados (`fn`, `pub fn`, métodos de traits).
  - Traits e tipos associados.
  - Constantes (`const`), estáticos (`static`) e apelidos de tipo (`type`).

### B. Estrutura Padrão de Documentação de Funções e Métodos
Toda função ou método não trivial deve conter as seguintes seções Rustdoc quando aplicável:
1. **Descrição Sumária:** Frase clara no imperativo ou presente explicando o efeito da função.
2. **Fundamentação Matemática / Física (se aplicável):**
   - Equações diferenciais, matrizes de transformação, perfis de cor ou integrais analíticas formatadas em KaTeX/LaTeX (`$...$` ou `$$...$$`) e representação textual.
   - Referência explícita aos presets do macOS Tahoe 26 (ex: $\zeta = 1.0$, $\omega_0 = \sqrt{k/m}$).
3. **Parâmetros (`Arguments`):** Significado físico e unidades de medida (ex: pixels, segundos, radianos, coordenadas locais de superfície).
4. **Erros (`# Errors`):** Lista exaustiva das condições que provocam o retorno de `Err(RendererError::...)` ou equivalente.
5. **Pânico (`# Panics`):** Garantia explícita de ausência de pânico em código de produção do compositor.

### C. Documentação de Shaders GLSL 450
- Todo arquivo GLSL em `crates/qz-renderer/shaders/` deve conter:
  - Cabeçalho explicando a finalidade visual e física (ex: Dual-Kawase, SDF Squircle $G^2$, Lente Refrativa).
  - Documentação de cada `layout(location = ...)` de entrada e saída.
  - Documentação detalhada dos blocos `layout(push_constant)` e uniformes de textura `layout(binding = ...)`.
  - Comentários passo a passo numerados correspondendo aos estágios analíticos de renderização (1. Difração/Refração, 2. Dispersão Cromática, 3. Boost de Saturação Rec.709, 4. Vibrancy Blend, 5. Specular Rim, 6. Dithering).

### D. Manutenção Viva do `CHANGELOG.md`
- Toda alteração que adicione, modifique, deprecie ou corrija comportamento no repositório **deve obrigatoriamente atualizar o arquivo `CHANGELOG.md`** na raiz do workspace, seguindo o padrão internacional [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).
- As seções padrão são:
  - `Added` (Novas funcionalidades e novos subsistemas).
  - `Changed` (Modificações em comportamentos existentes ou refatorações).
  - `Deprecated` (Funcionalidades marcadas para remoção futura).
  - `Removed` (Funcionalidades ou código legado excluído).
  - `Fixed` (Correções de bugs ou ajustes em testes/compilação).
  - `Security` (Melhorias de segurança, integridade de memória e tratamento de erros).

---

## 3. Versionamento Semântico Estrito (SemVer 2.0.0)

O projeto adota rigorosamente a especificação [Semantic Versioning 2.0.0](https://semver.org/lang/pt-BR/):

$$\text{VERSÃO} = \text{MAJOR}.\text{MINOR}.\text{PATCH}[-\text{PRERELEASE}]$$

### A. Regras de Incremento
1. **MAJOR (`X.0.0`):**
   - Incrementado quando ocorrem quebras incompatíveis na API pública de crates fundamentais, mudanças radicais na arquitetura de protocolos Wayland que quebrem clientes existentes, ou na transição da versão pré-1.0 para a versão `1.0.0` estável (produção completa).
2. **MINOR (`0.Y.0` ou `X.Y.0`):**
   - Incrementado a cada conclusão de uma das 17 fases ou subsistemas obrigatórios do macOS Tahoe 26 (ex: Fase 1 $\to$ `0.1.0-alpha.1`, Fase 2 $\to$ `0.2.0-alpha.1`, Fase 3 $\to$ `0.3.0-alpha.1`).
   - Adição de novos protocolos Wayland, novos shaders, novas crates ou novos recursos retrocompatíveis.
3. **PATCH (`0.Y.Z` ou `X.Y.Z`):**
   - Incrementado para correções de bugs retrocompatíveis, ajustes matemáticos de precisão em shaders, otimizações de performance sem alteração de API e correções de avisos de compilação.
4. **Pre-release Identifier (`-alpha.N`, `-beta.N`, `-rc.N`):**
   - Durante o desenvolvimento ativo do monorepo pré-1.0, todas as versões devem ser marcadas com o identificador de pré-lançamento adequado (ex: `0.1.0-alpha.1`, `0.2.0-alpha.1`).

### B. Sincronização de Versões no Workspace
- O arquivo raiz [`Cargo.toml`](file:///home/nepo/Downloads/MacTux/Cargo.toml) define `[workspace.package.version]`.
- Cada crate filha deve herdar a versão via `version.workspace = true` ou manter paridade exata com o marco arquitetural em desenvolvimento.
- Toda versão formal do workspace deve gerar uma Git Tag anotada no formato `vX.Y.Z-prerelease`.

---

## 4. Política de Git, Branches e Commits Convencionais

### A. Estratégia de Branches
O repositório utiliza o fluxo de desenvolvimento baseado em branches temáticos e integração contínua (Trunk-Based Development com Feature Branches):

| Prefixo de Branch | Finalidade | Exemplo |
|---|---|---|
| `main` | Ramo principal protegido. Código compilável, estável e testado. | `main` |
| `feat/*` | Desenvolvimento de nova fase, crate ou recurso do desktop. | `feat/phase-3-compositor-core` |
| `fix/*` | Correção de defeito, falha de renderização ou bug de protocolo. | `fix/squircle-sdf-gradient-epsilon` |
| `docs/*` | Documentação, diretrizes de engenharia, diagramas e changelog. | `docs/engineering-standards-and-versioning` |
| `refactor/*` | Melhoria interna de código sem alteração de comportamento externo. | `refactor/split-dma-modifiers` |
| `perf/*` | Otimização de pipeline gráfico, frame pacing ou alocação de memória. | `perf/vulkan-command-buffer-recycling` |
| `test/*` | Expansão de suíte de testes unitários ou de integração. | `test/spring-c2-numerical-convergence` |
| `chore/*` | Manutenção de dependências, scripts de automação ou CI. | `chore/update-github-actions-runners` |

### B. Especificação de Commits Convencionais (Conventional Commits 1.0.0)
Toda mensagem de commit deve seguir estritamente o formato:

```text
<tipo>(<escopo>): <descrição no imperativo e concisa>

[corpo opcional explicando motivação técnica, física e impacto na arquitetura]

[rodapé opcional referenciando issues, tags ou breaking changes]
```

#### Tipos Permitidos:
- `feat`: Novo subsistema, pipeline gráfico, protocolo ou funcionalidade.
- `fix`: Correção de bug em solver, shader, keymapper ou compositor.
- `docs`: Documentação, Rustdoc, especificações técnicas e CHANGELOG.
- `test`: Inclusão ou ajuste de testes automatizados.
- `perf`: Otimização de tempo de quadro, latência de input ou uso de GPU.
- `refactor`: Reestruturação de código sem adição de recurso ou correção de bug.
- `ci`: Alterações no pipeline de compilação remota do GitHub Actions.
- `chore`: Atualizações de dependências, `.gitignore` ou configurações de build.

#### Escopos Padronizados do Monorepo:
`animation`, `renderer`, `compositor`, `shell`, `spotlight`, `finder`, `quicklook`, `screenshot`, `daemons`, `session-lock`, `toolkit`, `portal`, `protocol`, `ci`, `standards`.

---

## 5. Checklist Mandatório de Auto-Aplicação a Cada Alteração

Antes de considerar qualquer alteração concluída, o engenheiro (IA) deve executar rigorosamente este checklist de 9 passos:

1. **Isolamento de Ramo:** Criar ou comutar para o branch semântico apropriado (`feat/*`, `fix/*`, `docs/*`).
2. **Código Zero-Tolerance:** Certificar-se de que não existem `// TODO`, `todo!()`, `unimplemented!()` ou blocos omitidos com `...`.
3. **Documentação Completa:** Garantir que todos os itens novos e alterados possuam Rustdoc completo em conformidade com `#![deny(missing_docs)]`.
4. **Fidelidade Matemática:** Confirmar que as equações analíticas, shaders e bindings físicos correspondem exatamente à especificação do macOS Tahoe 26.
5. **Testes Unitários:** Adicionar ou atualizar testes que verifiquem a invariância dos dados e a convergência matemática.
6. **Atualização do `CHANGELOG.md`:** Registrar detalhadamente a alteração na seção apropriada da versão corrente.
7. **Verificação Local:** Executar `cargo test --workspace` garantindo **zero falhas e zero avisos (`warnings`)**.
8. **Commit Convencional e Tag SemVer:** Criar commits atômicos padronizados e tags de versão anotadas se um marco foi atingido.
9. **Publicação e Validação em Nuvem:** Enviar (`git push`) o branch e as tags para o GitHub, aguardar a execução do workflow de CI e certificar-se do status `✓ SUCCESS` no runner remoto.

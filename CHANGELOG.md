# Changelog

Todos os commits significativos e mudanças de funcionalidades seguem o formato [Conventional Commits][1].
Este changelog acompanha as versões principais e secundárias do projeto LuwiScript.

---

## [Unreleased]

### Features
- Adicionada estrutura inicial de diretórios para o compilador (`compiler/`) e runtime (`runtime/`).
- Adicionado suporte básico a `libs/` (biblioteca padrão em LuwiScript) e `examples/`.
- Adicionado módulo de testes (`tests/unit`, `tests/integration`, `tests/acceptance`).

### Fixes

### Performance

### Refactor

---

## [v0.1.0] – 2026‑04‑19

### Features
- Inicialização do projeto `luwi-script` com submódulo `compiler` (Rust, C/C++).
- Implementação básica da estrutura de diretórios do compilador:
  - `ast/`, `parser/`, `typechecker/`, `codegen/`, `driver/`, `diagnostics/`.
- Implementação da estrutura de diretórios do runtime:
  - `vm/`, `scheduler/`, `stdlib/`.
- Adição de módulo de ferramentas:
  - `toolchain/fmt` (formatador) e `toolchain/linter` (linter).
- Criação do arquivo `LICENSE` sob licença MIT.
- Criação do arquivo `README.md` com visão geral do projeto.

### Breaking changes
- Nenhuma nessa versão inicial.

### Deprecations
- Nenhuma nessa versão inicial.

---

## Versioning

As versões seguem o padrão **SemVer**: `MAJOR.MINOR.PATCH`, por exemplo: `v0.1.0`.
Alguns exemplos de significado:

- `v0.1.0` – versão inicial funcional.
- `v0.2.0` – novas features importantes, sem quebras compatíveis.
- `v1.0.0` – API estável para produção.

[1]: https://www.conventionalcommits.org

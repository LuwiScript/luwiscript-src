# LuwiScript

**LuwiScript** é uma linguagem de programação estática e orientada a sistemas corporativos, projetada para rodar em qualquer sistema operacional moderno (Linux, macOS, Windows e FreeBSD).
O compilador é escrito principalmente em Rust, com partes críticas em C/C++, enquanto a biblioteca padrão é escrita totalmente em LuwiScript.

---

## 📌 Objetivos

- Proporcionar uma linguagem **clara, segura e eficiente** para sistemas de backend corporativos, APIs, automação e processos internos de empresas.
- Oferecer um compilador **portável e estável**, com geração de código nativo ou bytecode, integrado a um runtime multi‑plataforma.
- Manter uma **biblioteca padrão rica**, mas modular, focada em I/O, coleções, concorrência e integrações comuns (arquivos, rede, APIs, bancos de dados).

---

## ✨ Características principais

- Sintaxe limpa e legível, inspirada em linguagens modernas.
- Tipagem estática com inferência parcial e checagem de erros rica.
- Suporte a **concorrência leve** (corotinas / tarefas cooperativas) via runtime embutido.
- Compilador modular:
  - `parser` → `typechecker` → `codegen` → `driver`.
- Biblioteca padrão (`libs/std/`) escrita completamente em LuwiScript.
- Roda em:
  - Linux, macOS, Windows e FreeBSD.
- Ferramentas de desenvolvimento:
  - `luwfmt` (formatador de código).
  - `luwlint` (linter integrado ao compilador).

---

## 📦 Estrutura do projeto

```text
luwi-script/
├── LICENSE                             # MIT License
├── README.md                           # Este arquivo
├── docs/                               # Documentação da linguagem e compilador
├── compiler/                           # Compilador (Rust + C/C++)
└── runtime/                            # Runtime (VM, scheduler, stdlib)
└── libs/                               # Bibliotecas padrão escritas em LuwiScript
└── examples/                           # Exemplos de código
└── tests/                              # Testes (unit, integration, acceptance)
└── toolchain/                          # Ferramentas (luwfmt, luwlint)
```

---

## 🛠️ Como construir e instalar (desenvolvedor)

### Requisitos

- Rust (Nightly ou estável, conforme o `rust-toolchain`).
- LLVM ≥ 14 (se usar geração de código via LLVM).
- `make`/`cargo` disponíveis.

### Passos básicos

```bash
# Clonar o repositório
git clone https://github.com/seu-usuario/luwi-script.git
cd luwi-script

# Construir o compilador
cargo build --bin luwic --workspace

# Instalar (exemplo manual, você pode criar um script de instalação)
cp target/debug/luwic /usr/local/bin/luwic
```

Após isso, você pode compilar um script de exemplo:

```bash
luwic examples/hello.lw -o hello
./hello
```

---

## 📖 Documentação

Veja a pasta `docs/`:

- `language-spec.md` – Manual da linguagem (tipos, sintaxe, concorrência).
- `stdlib-reference.md` – Referência da biblioteca padrão.
- `compiler-design.md` – Visão de alto nível do compilador e pipeline.
- `propostas-novidades.md` – Roadmap sugerido de funcionalidades para evolução da linguagem.

---

## 🧪 Testes

Rodar testes unitários:

```bash
cargo test --test unit
```

Rodar testes de integração:

```bash
cargo test --test integration
```

Rodar testes de aceitação (casos de uso corporativos):

```bash
cargo test --test acceptance
```

---

## 🧰 Ferramentas de desenvolvimento

- `luwfmt` – Formatador de código de LuwiScript.

  ```bash
  cargo run --bin luwfmt -- src/**/*.lw
  ```

- `luwlint` – Linter de estilo, segurança e padrões de sistema corporativo.

  ```bash
  cargo run --bin luwlint -- src/**/*.lw
  ```

---

## 📄 Licença

Este projeto está sob a licença **MIT** – veja o arquivo [`LICENSE`](LICENSE) para mais detalhes. [web:5][web:95]

---

## 🤝 Contribuição

Contribuições são bem‑vindas!
Você pode:

- Abrir issues com ideias de melhorias ou bugs.
- Enviar PRs com novas funcionalidades ou otimizações no compilador.
- Propor novas funções na biblioteca padrão, sempre pensando em sistemas corporativos.

---

> **LuwiScript · 2026 · Lucas Gabriel Witchemichen**

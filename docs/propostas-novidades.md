# Propostas de novidades para LuwiScript

Este documento lista funcionalidades sugeridas para evoluir a linguagem LuwiScript, com foco em produtividade, segurança e sistemas corporativos.

## 1) `match` com verificação de exaustividade

**Proposta:** adicionar expressão/comando `match` para `enum`, literais e intervalos.

**Valor:**
- reduz cadeias longas de `if/else`;
- melhora legibilidade em regras de negócio;
- permite diagnósticos de casos não tratados em tempo de compilação.

## 2) `Result<T, E>` e operador `?`

**Proposta:** padronizar tratamento de erro com `Result` e propagação via `?`.

**Valor:**
- modelo explícito de erro para integrações de I/O, rede e banco;
- simplifica funções com múltiplas etapas;
- favorece APIs previsíveis no ecossistema.

## 3) Pattern matching em `let`, parâmetros e loops

**Proposta:** ampliar patterns destruturáveis para arrays, structs e enums.

**Valor:**
- extração de dados mais concisa;
- menos código boilerplate;
- alinhamento com a AST já orientada a `Pattern`.

## 4) Generics em funções, structs e enums

**Proposta:** suportar parâmetros de tipo (`fn map<T, U>(...)`).

**Valor:**
- reutilização forte de código;
- coleções e utilitários de stdlib mais expressivos;
- menos duplicação em código corporativo.

## 5) Traits/interfaces e `impl` com constraints

**Proposta:** modelo de contratos para polimorfismo estático (`trait`, `impl`, `where`).

**Valor:**
- design mais modular;
- permite ecossistema de bibliotecas extensível;
- prepara o caminho para frameworks internos.

## 6) Imutabilidade por padrão + `mut`

**Proposta:** variáveis imutáveis por padrão e mutabilidade explícita.

**Valor:**
- evita classes de bugs por alteração acidental;
- simplifica raciocínio em concorrência;
- fortalece segurança sem custo de runtime.

## 7) Módulos com visibilidade (`pub`, `pub(crate)`)

**Proposta:** formalizar sistema de módulos, import/export e níveis de visibilidade.

**Valor:**
- organização de projetos médios/grandes;
- encapsulamento de detalhes internos;
- melhor DX para toolchain e IDE.

## 8) Async/await integrado ao runtime

**Proposta:** adicionar `async fn`, `await` e tarefas com cancelamento cooperativo.

**Valor:**
- I/O concorrente sem bloquear;
- APIs web e jobs assíncronos mais simples;
- aproveita o scheduler já existente no runtime.

## 9) Coleções avançadas na stdlib

**Proposta:** incluir `HashMap`, `HashSet`, `VecDeque`, `BTreeMap` e iteradores ricos.

**Valor:**
- remove necessidade de soluções ad hoc;
- aumenta performance e clareza em cenários comuns;
- melhora adoção da linguagem em produção.

## 10) Atributos/anotações pragmáticas

**Proposta:** expandir atributos (`@deprecated`, `@inline`, `@test`, `@derive(...)`).

**Valor:**
- metaprogramação leve sem macros completas no início;
- governança de APIs internas;
- base para automações de build/test.

## 11) Interpolação de strings

**Proposta:** strings com placeholders (`"pedido {id} status {status}"`).

**Valor:**
- logging e mensagens de erro mais limpas;
- menos concatenação manual;
- ganho direto de produtividade.

## 12) Melhorias de diagnósticos do compilador

**Proposta:** mensagens com sugestões de correção, notas contextuais e códigos de erro.

**Valor:**
- onboarding mais rápido;
- redução do custo de manutenção;
- integração melhor com linter e IDE.

## Priorização sugerida (MVP → maturidade)

1. `Result<T,E>` + `?`
2. `match` exaustivo
3. módulos/visibilidade
4. imutabilidade por padrão + `mut`
5. generics
6. traits/interfaces
7. async/await
8. atributos e stdlib avançada

## Notas de implementação no projeto atual

- O parser já possui estrutura para `Pattern`, `Enum`, `Struct` e `Impl`, o que facilita iniciar por `match` e destruturação.
- O runtime já possui scheduler, reduzindo atrito para recursos assíncronos.
- A presença de `typechecker` modular favorece introdução incremental de generics e constraints.

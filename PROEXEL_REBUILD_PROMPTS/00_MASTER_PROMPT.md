# MASTER PROMPT — Rebuild completo do PROEXEL

Você está reconstruindo o PROEXEL do zero.

## Contexto de repositórios

Você terá acesso a três árvores de código:

1. **AppCore-Runtime** — runtime local-first genérico em Rust. É infraestrutura congelada/contratos-first. Lógica do PROEXEL não pertence dentro dos crates AppCore.
2. **admin-dashboard-templ** — template Next.js/React/shadcn que será a base visual e estrutural do frontend.
3. **PROEXEL antigo** — React/Vite/Supabase. Use-o somente para descobrir regras de negócio, fluxos, dados, permissões, assets e comportamento esperado.

Se os caminhos reais forem diferentes, localize os três repositórios pela raiz e documente os caminhos encontrados. Não peça ao usuário informações que podem ser obtidas lendo os repositórios.

## Missão

Criar uma nova aplicação PROEXEL, limpa e moderna, usando AppCore como runtime/base de infraestrutura e o admin-dashboard como UI. Preserve a lógica útil do produto, mas **não preserve a arquitetura antiga**.

## Proibições

- Não transformar o PROEXEL antigo incrementalmente.
- Não copiar `App.jsx`, hooks `useSupabase`, SCSS antigo ou componentes JSX antigos como base.
- Não criar lógica de negócio dentro de `AppCore-Runtime/crates/*`.
- Não acoplar a UI diretamente ao banco.
- Não usar Supabase como fonte primária do novo domínio apenas porque o legado usa.
- Não implementar “offline” como simples fallback silencioso em localStorage.
- Não inventar regras quando o código antigo pode ser consultado.
- Não substituir regras existentes por mocks para declarar conclusão.

## Princípios arquiteturais

- AppCore permanece product-independent.
- PROEXEL é uma aplicação consumidora, com domínio próprio.
- Local-first de verdade: a operação primária deve continuar disponível localmente quando coerente com o papel/permissão e modo operacional.
- Separar `domain`, `application`, `infrastructure/adapters` e `ui`.
- Definir comandos e queries explícitos; writes devem ser idempotentes onde houver possibilidade de retry/sync.
- Persistência deve ter schema versionado e migrações.
- Sync é uma preocupação de infraestrutura; conflito e ownership precisam ser explícitos.
- A UI conhece contratos da aplicação, não detalhes do AppCore nem do storage.
- Auditoria é transversal e não pode depender da UI lembrar de registrar cada operação manualmente.

## Comportamento funcional mínimo que deve sobreviver

### Válvulas
Campos equivalentes a TAG, zona, marca/fabricante, série, kit, assento, DN, tipo, atuador, fabricação, última troca de kit e última manutenção. TAG deve ser normalizada e pesquisável.

### Saúde/manutenção
Regra atual a preservar inicialmente:
- sem `ultima_manutencao` => crítico;
- > 180 dias => crítico;
- > 150 dias => atenção;
- <= 150 dias => OK.

Centralize isso em uma policy de domínio testável; não deixe a regra espalhada na UI.

Manutenção registra pelo menos: válvula/TAG, data, técnico, tipo preventiva/corretiva, serviço, notas, assinatura e se houve troca de kit.

Se houve troca de kit e existir item de estoque compatível, debitar exatamente 1 unidade de forma transacional/idempotente. Não permitir estoque negativo. O resultado deve ser auditável.

### OS / Agenda
Ordem de serviço: zona, válvula opcional, descrição, prioridade, status, criador, técnico e data programada. Preservar estados equivalentes a pendente/aberta, andamento e concluída, normalizando o legado.

### Reposição
Solicitação de reposição: referência, motivo/descrição, solicitante e status pendente/aprovada/rejeitada.

### Estoque
Referência única normalizada, quantidade >= 0, mínimo >= 0, fabricante e/ou localização como campos separados. Corrigir a ambiguidade do legado em que `brand` e `location` são intercambiáveis.

### Fornecedores
Nome, contato, email, website, notas, criado por e timestamps.

### Fotos
Foto da válvula associada de forma estável ao ID da válvula; TAG é atributo mutável e não deve ser a chave interna definitiva.

### Auditoria
Registrar ator, papel, operação, agregado, id, before/after quando apropriado, descrição, timestamp, trace/correlation id. A auditoria não deve bloquear a operação principal se o sink secundário falhar, mas o evento local deve ser persistido com confiabilidade.

### RBAC
Papéis do legado: admin, chefe, compras, tecnico. Extraia a matriz real do código antes de consolidar permissões. Permissões devem existir no backend/application layer; esconder botão não é segurança.

### Relatórios
Preservar relatórios geral, por zona, válvulas críticas e manutenções recentes. A apresentação pode mudar completamente para combinar com a nova UI.

### i18n
Preservar PT/EN/ES, removendo strings hardcoded importantes durante o rebuild.

## UI

Use o `admin-dashboard-templ` como base real:
- Next.js + React + TypeScript;
- sidebar, header, layout controls, themes e componentes shadcn existentes;
- tabelas TanStack, formulários tipados, dialogs/sheets, cards, tabs, date pickers e charts quando agregarem valor.

Remova páginas demo e navegação irrelevante. A nova navegação deve refletir PROEXEL, por exemplo:
- Visão geral
- Válvulas
- Manutenção
- Ordens / Agenda
- Estoque
- Compras / Reposição
- Fornecedores
- Histórico / Auditoria
- Relatórios
- Administração (conforme papel)

Não tente reproduzir o visual neon/industrial antigo. Preserve a informação e os fluxos, mas use o design system do dashboard.

## Processo obrigatório

1. Faça discovery do legado e produza `docs/legacy-behavior-map.md`.
2. Faça um ADR de arquitetura-alvo.
3. Crie o novo app em diretório/repositório próprio, não em cima do legado.
4. Implemente domínio e persistência antes de conectar telas complexas.
5. Faça cada vertical slice funcional ponta a ponta.
6. Implemente migração/importação do legado separadamente.
7. Rode build, lint, testes e, se disponível no workspace, gates do AppCore relevantes ao consumidor.
8. Não declare finalizado enquanto houver página principal só com mock.

## Definition of Done

A reconstrução está pronta quando:
- o legado não é dependência de runtime;
- AppCore não contém regra PROEXEL;
- todas as funções principais possuem fluxo real de leitura/escrita;
- RBAC é aplicado fora da UI;
- regras de manutenção/estoque possuem testes;
- banco/schema é versionado;
- migração é repetível e auditável;
- UI é coerente com admin-dashboard e responsiva;
- PT/EN/ES funcionam nas áreas principais;
- build/lint/tests passam;
- existe documentação de execução, configuração, backup e migração.

Ao final de cada etapa, escreva um resumo curto com arquivos alterados, testes executados, decisões tomadas e pendências reais. Continue implementando; não pare apenas para produzir um plano.

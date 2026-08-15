# PROEXEL Rebuild — Prompt Pack

Este pacote foi preparado a partir de três fontes concretas:

- `AppCore-Runtime-main`: arquitetura/runtime local-first. Deve ser **consumido**, não modificado para acomodar lógica do PROEXEL.
- `admin-dashboard-templ-main`: referência/base visual Next.js + React + shadcn para a nova UI.
- `PROEXEL-main`: aplicação antiga. Deve ser tratada apenas como **especificação executável do comportamento**, fonte de dados, regras e assets úteis. Não reutilizar a arquitetura antiga.

## Objetivo

Refazer completamente o PROEXEL, preservando apenas sua lógica funcional e seus dados relevantes. O novo produto deve ser uma aplicação externa ao AppCore, com domínio próprio e UI baseada no admin dashboard.

## Como usar

Execute os prompts em ordem. O `00_MASTER_PROMPT.md` pode ser usado sozinho para uma execução longa; para maior controle, use `01` a `11` sequencialmente. Em cada etapa, o agente deve primeiro ler o código necessário, depois implementar, testar e documentar o que mudou.

### Ordem sugerida

1. `00_MASTER_PROMPT.md` — contrato global e regras inegociáveis.
2. `01_DISCOVERY_AND_BEHAVIOR_MAP.md` — inventário verificável da aplicação antiga.
3. `02_TARGET_ARCHITECTURE.md` — desenho da arquitetura externa ao AppCore.
4. `03_BOOTSTRAP_NEW_APP.md` — novo workspace e integração inicial.
5. `04_DOMAIN_AND_STORAGE.md` — domínio, schema e persistência local-first.
6. `05_AUTH_RBAC_AUDIT.md` — autenticação, papéis e auditoria.
7. `06_UI_ADMIN_DASHBOARD.md` — reconstrução total da interface.
8. `07_MAINTENANCE_VALVES.md` — válvulas/manutenção/fotos/assinatura.
9. `08_ORDERS_STOCK_PURCHASING.md` — OS, estoque, reposição e fornecedores.
10. `09_REPORTS_I18N_OFFLINE.md` — relatórios, idiomas e experiência offline.
11. `10_MIGRATION_AND_COMPATIBILITY.md` — importação/migração do legado.
12. `11_TESTS_HARDENING_RELEASE.md` — testes, segurança, qualidade e release.

## Regra central

**Não fazer um refactor do PROEXEL antigo. Fazer um produto novo.** O legado é uma referência comportamental. Não carregar Vite, hooks Supabase, estado global no `App.jsx`, CSS/SCSS antigo, componentes antigos ou fallback de localStorage como arquitetura final.

## Estado de execução

Os prompts 01, 02, 03, 04, 05, 08 e 09 estão marcados no próprio título como
`CONCLUÍDO 100%`. Os prompts 00, 06, 07, 10 e 11 permanecem sem essa marca porque
possuem requisitos literais ainda abertos ou evidência externa de produção ainda
indisponível. O detalhamento verificável fica em
`docs/implementation-status.md` e `docs/functional-parity-checklist.md`.

## Regras de negócio já identificadas no legado

- Entidade de válvula com: TAG, zona, fabricante/marca, série, kit, assento, DN, tipo, atuador, fabricação, última troca de kit e última manutenção.
- Status de manutenção: sem última manutenção = crítico; acima de 180 dias = crítico; acima de 150 dias = atenção; demais = OK.
- Dashboard agrega válvulas por status e zona e mostra manutenção recente.
- Manutenção pode ser preventiva ou corretiva e guarda técnico, serviço, notas, assinatura e indicação de troca de kit.
- Quando uma manutenção troca um kit, existe baixa automática de 1 unidade do estoque quando a referência correspondente existe.
- Ao cadastrar válvula com kit, o legado garante uma referência de estoque correspondente, inicialmente com quantidade 0 quando necessário.
- Ordens de serviço têm zona, válvula opcional, descrição, prioridade, status, criador, técnico e data programada.
- Solicitações de reposição possuem referência/kit, descrição/motivo, solicitante, status pendente/aprovada/rejeitada.
- Estoque tem referência do kit, quantidade, quantidade mínima e fabricante/localização (o legado mistura os nomes; no novo domínio isso deve ser normalizado).
- Fornecedores têm nome, contato, email, website, notas e autor da criação.
- Fotos de válvulas são associadas por TAG.
- Auditoria registra usuário, papel, ação, tabela/agregado, record id, valores antigos/novos, descrição e timestamp.
- Papéis existentes: `admin`, `chefe`, `compras`, `tecnico`.
- Relatórios atuais incluem visão geral, por zona, críticos e histórico recente.
- Idiomas existentes no legado: português, inglês e espanhol.

Essas regras devem ser confirmadas contra o código antes de qualquer migração destrutiva.

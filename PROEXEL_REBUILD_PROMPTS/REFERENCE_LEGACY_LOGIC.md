# Referência rápida - lógica observada no PROEXEL antigo - CONCLUÍDO 100%

Este arquivo é uma orientação para o agente, não substitui a leitura do código.

## Fonte principal de orquestração
`src/App.jsx` concentra views, hooks de dados, permissões, status, side effects e modais. Isso é um anti-pattern a não carregar para o rebuild.

## Data access antigo
`src/lib/useSupabase.js` contém hooks para:
- login via `app_login`;
- valves;
- maintenance_records;
- orders;
- restock_requests;
- stock;
- suppliers;
- audit_log;
- valve photos/storage.

Há normalizações ad hoc de snake_case/camelCase, forte sinal de que o novo schema deve ser canônico.

## Maintenance status
No `App.jsx`, a regra usa diferença em dias desde `ult_man`:
- ausência => `crit`;
- >180 => `crit`;
- >150 => `warn`;
- caso contrário => `ok`.

## Side effects importantes
- Registrar manutenção grava histórico.
- Se `kitChanged` e houver kit/stock correspondente, reduz 1 unidade do estoque.
- `changeStockQuantity` usa `max(0, ...)`, portanto o legado evita negativo silenciosamente.
- Criar válvula com kit tenta garantir registro de stock com qty 0/min 1.

## Papéis / views observados
No `App.jsx`:
- admin: dash, valves, agenda, painel, compras, historico
- chefe: dash, valves, agenda, painel, compras, historico
- compras: compras
- tecnico: valves, agenda

Há permissões mais específicas dentro dos componentes; o novo sistema deve mapear ações, não apenas páginas.

## Agenda
O componente antigo mistura:
- OS;
- calendário/prioridade de manutenção;
- sugestão de reposição feita por técnico;
- revisão de reposição por papéis superiores/compras.

No rebuild, separar conceitualmente sem perder o fluxo.

## PDF
O relatório antigo calcula total/OK/warn/crit, resumo por zona, válvulas críticas e manutenção recente. O novo relatório deve usar queries do backend em vez de duplicar a policy na UI.

## UI
O visual antigo é custom React/Vite/SCSS com overlays e estilo industrial neon. Não preservar. O admin-dashboard Next/shadcn é a nova linguagem visual.

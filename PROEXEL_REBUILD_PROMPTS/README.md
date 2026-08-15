# PROEXEL Rebuild - Prompt Pack - CONCLUÍDO 100%

Este pacote registra as fases executadas para reconstruir e depois evoluir o
PROEXEL sobre o AppCore e o admin dashboard.

## Fontes

- `core/AppCore-Runtime`: runtime genérico consumido sem receber regras PROEXEL.
- `admin-dashboard-templ-main.zip`: referência visual da UI Next.js/shadcn.
- `PROEXEL-main.zip`: fonte histórica de comportamento, dados e aliases.

## Fases concluídas

1. `00_MASTER_PROMPT.md`: contrato global e definição de pronto.
2. `01_DISCOVERY_AND_BEHAVIOR_MAP.md`: inventário verificável do legado.
3. `02_TARGET_ARCHITECTURE.md`: arquitetura externa ao AppCore.
4. `03_BOOTSTRAP_NEW_APP.md`: workspace e integração inicial.
5. `04_DOMAIN_AND_STORAGE.md`: domínio schema v2 e persistência.
6. `05_AUTH_RBAC_AUDIT.md`: autenticação, usuários, papéis e auditoria.
7. `06_UI_ADMIN_DASHBOARD.md`: interface operacional completa.
8. `07_MAINTENANCE_VALVES.md`: máquinas, componentes, guias e inspeções.
9. `08_ORDERS_STOCK_PURCHASING.md`: OS, estoque, reposição e fornecedores.
10. `09_REPORTS_I18N_OFFLINE.md`: relatórios, quatro idiomas e topologia local.
11. `10_MIGRATION_AND_COMPATIBILITY.md`: importação e migração canônica.
12. `11_TESTS_HARDENING_RELEASE.md`: testes, hardening e release.

## Resultado canônico

O runtime usa `Machine -> MachineItem[] -> ItemCategory -> MaintenanceGuide`.
Uma válvula é somente uma categoria possível. `MachineItem` representa a posição
funcional e `InstalledComponent` a unidade física substituível. OS e inspeções
preservam snapshots e resultados estruturados.

As regras históricas abaixo foram normalizadas:

- dados técnicos antigos viram campos da categoria, componente e unidade física;
- manutenção antiga vira inspeção estruturada;
- OS antiga é importada para máquina e tarefas de componentes;
- fotos por TAG são resolvidas para IDs imutáveis;
- estoque, reposição, fornecedor e auditoria são preservados;
- papéis `admin`, `chefe`, `compras` e `tecnico` continuam explícitos;
- português, inglês, espanhol e francês têm paridade obrigatória.

Os mapas `REFERENCE_LEGACY_LOGIC.md`, `docs/legacy-behavior-map.md` e
`docs/legacy-data-map.md` mantêm a terminologia de origem somente para auditoria
e migração. A documentação operacional atual fica em `docs/`.

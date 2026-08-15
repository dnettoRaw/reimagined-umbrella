# Prompt 02 - Arquitetura-alvo PROEXEL + AppCore - CONCLUÍDO 100%

## Decisão entregue

AppCore continua sendo runtime genérico. Machine, MachineItem, ItemCategory,
MaintenanceGuide, ServiceOrder, ItemInspection, estoque, compras e usuários são
conceitos exclusivos do workspace `proexel/`.

## Camadas

- `proexel-domain`: entidades, value objects, snapshots e policies.
- `proexel-application`: commands, queries, RBAC, transações e auditoria.
- `proexel-infrastructure`: persistência JSON atômica e anexos protegidos.
- `proexel-migration`: importação legada determinística.
- `apps/service`: composition root hospedado pelo AppCore.
- `apps/web`: sessão, proxy de capabilities e UI administrativa.

## Modelo final

`MachineItem` é a posição funcional estável. `InstalledComponent` é a unidade
física atual. `MachineItemReplacement` preserva cada troca. `ItemCategory` define
campos dinâmicos e o guia versionado. OS e inspeções mantêm snapshots suficientes
para que alterações futuras não reescrevam o contexto histórico.

## Dependências

As dependências apontam de infraestrutura/serviço para aplicação e domínio. Um
teste de boundary impede que domínio/aplicação dependam de AppCore ou da web.
Nenhum crate genérico do AppCore recebeu regra de negócio PROEXEL.

## Persistência e topologia

O estado canônico schema v2 é transacionado e auditado em arquivo local. O
schema v1 é migrado automaticamente com backup. A topologia suportada é local
read/write; sync remoto não é simulado.

Referência detalhada: `docs/architecture.md` e
`docs/appcore-integration-surface.md`.

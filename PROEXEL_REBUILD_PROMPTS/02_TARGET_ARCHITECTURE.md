# Prompt 02 — Arquitetura-alvo PROEXEL + AppCore — CONCLUÍDO 100%

Com base no discovery, desenhe e aplique a arquitetura-alvo do novo PROEXEL.

## Restrição principal

O AppCore é runtime genérico. Não adicione `proexel-*`, Valve, Maintenance, Stock, Supplier ou outra regra de negócio em `AppCore-Runtime/crates`.

## Estrutura desejada

Prefira um workspace próprio semelhante a:

```text
proexel/
  apps/
    web/                 # Next.js baseado no admin-dashboard
    service/             # composition root/backend da aplicação, se aplicável
  crates/
    proexel-domain/
    proexel-application/
    proexel-infrastructure/
    proexel-migration/
  packages/
    contracts/           # tipos/DTOs compartilhados com a UI, se necessário
  migrations/
  docs/
```

Adapte se houver uma razão concreta, mas preserve as fronteiras.

## Domínio

Defina agregados/value objects/enums para pelo menos:

- Valve
- MaintenanceRecord
- ServiceOrder
- RestockRequest
- StockItem
- Supplier
- User/Role/Permission (ou integração equivalente)
- ValvePhoto metadata
- AuditEvent

Não modele tudo como `serde_json::Value` ou mapas genéricos.

## Application layer

Crie comandos e queries explícitos, por exemplo:

- CreateValve / UpdateValve / GetValve / SearchValves
- RegisterMaintenance
- CreateServiceOrder / ChangeServiceOrderStatus
- CreateRestockRequest / ReviewRestockRequest
- AdjustStock / UpsertStockItem
- CreateSupplier / UpdateSupplier
- ListAuditEvents
- GenerateReportData

Os nomes podem variar, mas a separação deve existir.

## AppCore

Identifique e use apenas APIs públicas estáveis. Escreva `docs/adr/0001-appcore-consumer-boundary.md` explicando:

- o que PROEXEL delega ao runtime;
- o que continua sendo responsabilidade da aplicação;
- storage;
- audit/observability;
- sync;
- security/auth;
- health/lifecycle;
- update;
- deployment standalone/distributed, se aplicável.

Crie um `ApplicationManifestV1` válido para PROEXEL com capabilities funcionais, sem caminhos/segredos de instalação.

## Comunicação UI ↔ aplicação

A UI não deve importar crates Rust nem saber detalhes de banco. Defina uma API/transport local clara, versionada, com DTOs e erros tipados. Se o AppCore já expõe a composição HTTP adequada para consumidores, reutilize o contrato público em vez de inventar um framework paralelo.

## Qualidade

Implemente skeleton compilável, não apenas documentação. Adicione testes arquiteturais simples ou checks que reduzam risco de o domínio passar a depender da camada web/infrastructure.

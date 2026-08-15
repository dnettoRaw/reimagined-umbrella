# Prompt 04 — Domínio, banco e local-first — CONCLUÍDO 100%

Implemente a camada de domínio e persistência do PROEXEL novo.

## Schema canônico

Crie schema versionado/migrations. Use IDs internos estáveis (UUID/ULID ou estratégia coerente) em vez de usar TAG como primary key.

### valves
- id
- tag normalizada e unique dentro do escopo correto
- zone
- manufacturer
- serial_number
- kit_reference
- seat
- dn
- valve_type
- actuator_reference
- manufacturing_year
- last_kit_change_at
- last_maintenance_at
- created_at / updated_at

### maintenance_records
- id
- valve_id
- performed_at
- technician / technician_user_id quando possível
- type: preventive | corrective
- service_description
- notes
- kit_changed
- kit_reference_snapshot
- signature reference/data strategy
- created_at

### service_orders
- id
- zone
- valve_id nullable
- description
- priority
- status canônico
- created_by
- assigned_technician
- scheduled_for
- created_at / updated_at

### restock_requests
- id
- stock_item_id nullable
- requested_reference snapshot
- reason
- requested_by
- status pending | approved | rejected
- reviewed_by / reviewed_at
- created_at

### stock_items
- id
- reference unique normalizada
- manufacturer nullable
- location nullable
- quantity non-negative
- min_quantity non-negative
- created_at / updated_at

### suppliers
Campos funcionais equivalentes ao legado, com timestamps.

### valve_photos / attachments
Relacionar ao `valve_id`, metadados, hash/content type/size e storage reference.

### audit
Se AppCore fornece infraestrutura de audit genérica, integre. Ainda assim mantenha contrato do domínio para contexto semântico PROEXEL.

## Policies de domínio

Implemente e teste:

### MaintenanceStatusPolicy
- nunca mantida => Critical
- dias > 180 => Critical
- dias > 150 => Warning
- demais => Ok

Use relógio injetável/testável. Não use `Date.now()` espalhado.

### RegisterMaintenance
Operação atômica do ponto de vista da aplicação:
- cria registro;
- atualiza `last_maintenance_at` da válvula;
- se troca de kit, atualiza `last_kit_change_at`;
- se troca de kit e item existe, baixa 1;
- nunca deixa estoque negativo;
- gera auditoria/eventos correspondentes;
- retry não deve baixar o mesmo kit duas vezes.

### CreateValve
Se `kit_reference` existir, garanta item de estoque correspondente conforme a regra legada, mas faça isso explicitamente por command/domain service e de forma idempotente.

## Local-first

Não use localStorage como banco. Use storage local confiável através da arquitetura AppCore/aplicação e outbox/sync quando necessário. Documente ownership e o que acontece offline.

## Testes

Adicione testes unitários para todas as bordas de 150/180 dias, manutenção com/sem kit, estoque zero, retry/idempotência e normalização de referências.

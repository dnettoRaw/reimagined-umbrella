# Prompt 04 - Domínio e storage - CONCLUÍDO 100%

## Entidades entregues

- ItemCategory, CustomFieldDefinition, MaintenanceGuide e MaintenanceGuideStep.
- Machine, MachineItem, InstalledComponent e MachineItemReplacement.
- ReplacementSpecification e EquivalentPart.
- PhotoAsset com owner e purpose tipados.
- ServiceOrder, ServiceOrderTask e snapshots imutáveis.
- ItemInspection, InspectionStepResult e InspectionFinding.
- UserAccount com role, active, auth version e maximum repair level.
- Estoque, reposição, fornecedor e auditoria preservados.

## Policies entregues

- normalização e unicidade de códigos;
- complexidade 1..5;
- validação tipada de campos dinâmicos;
- elegibilidade técnica do operador;
- transições de OS;
- validação de guia, foto e medição;
- estoque não negativo;
- status de máquina derivado de itens;
- idempotência e auditoria atômicas.

## Storage entregue

`JsonFileStore` serializa concorrência, grava por arquivo temporário e rename,
preserva receipts e falha sem commit parcial. O decoder aceita schema v1, cria
backup e produz schema v2. Testes cobrem durabilidade, concorrência, falha e
migração.

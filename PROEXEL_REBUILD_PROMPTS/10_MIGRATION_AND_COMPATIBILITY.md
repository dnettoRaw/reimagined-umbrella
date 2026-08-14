# Prompt 10 — Migração do PROEXEL legado

Crie uma ferramenta de migração separada e repetível. O objetivo é preservar dados, não preservar infraestrutura Supabase/localStorage.

## Fontes possíveis

Com base no discovery, suporte importação dos datasets relevantes do legado:
- valves
- maintenance_records
- orders
- restock_requests
- stock
- suppliers
- valve photos metadata/files
- usuários apenas se houver estratégia segura de migração de identidade
- audit antigo opcional como histórico imutável

Também considere exports/localStorage apenas como fonte de recuperação, nunca como storage final.

## Normalizações obrigatórias

- `zona` / `zone` → campo canônico;
- `valveTag` / `valve_tag` → `valve_id` resolvido quando possível + snapshot;
- `observacoes` / `description`;
- `createdBy` / `created_by`;
- restock `ref` / `kit` e `description` / `reason`;
- `minQuantity` / `min_quantity`;
- **não** continuar equivalendo manufacturer/brand com location; resolva com heurística documentada ou campo de migration_note;
- status `aberta` versus `pendente`;
- `kitChanged` / `kit_changed`.

## Requisitos da ferramenta

- dry-run;
- relatório de contagens antes/depois;
- warnings por linha problemática;
- deterministic mapping;
- idempotência: rodar duas vezes não duplica;
- checksums/import batch id;
- transação por lote quando possível;
- arquivo de relatório final JSON + Markdown;
- nenhuma senha plaintext nos relatórios.

## Validação

Compare, por entidade:
- contagem total;
- referências únicas;
- vínculos manutenção→válvula;
- vínculos OS→válvula quando existentes;
- estoque;
- fotos encontradas/perdidas;
- registros não migráveis.

Crie `docs/migration-runbook.md` com backup, dry-run, execução, rollback e validação.

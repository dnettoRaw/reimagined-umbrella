# Prompt 01 — Discovery e mapa de comportamento — CONCLUÍDO 100%

Antes de escrever a nova aplicação, faça uma auditoria completa do PROEXEL legado e da superfície pública relevante do AppCore.

## Trabalho

Leia todo código funcional do PROEXEL antigo, ignorando assets binários quando não forem necessários. Mapeie:

- rotas/views e modais;
- entidades e campos;
- operações CRUD;
- regras derivadas/calculadas;
- status/enums implícitos;
- papéis e permissões;
- side effects entre módulos;
- fontes de dados Supabase/localStorage/storage;
- tabelas/RPC/buckets usados;
- comportamento offline;
- geração de PDF;
- fotos;
- assinatura;
- i18n;
- notificações;
- auditoria;
- qualquer compatibilidade de nomes snake_case/camelCase.

Confirme no código, entre outros, os usos de `valves`, `maintenance_records`, `orders`, `restock_requests`, `stock`, `suppliers`, `audit_log`, storage de fotos e RPC de login.

Leia também `AppCore-Runtime/docs/architecture.md`, `APPLICATION_MANIFEST.md`, `DEPLOYMENT_MANIFEST.md`, `PROVIDER_MODEL.md`, os READMEs de `appcore-api`, `appcore-contracts`, `appcore-types`, `appcore-storage`, `appcore-sync`, `appcore-security` e demais crates que forem realmente relevantes ao consumidor.

## Artefatos que você deve criar no NOVO repositório

Crie:

- `docs/legacy-behavior-map.md`
- `docs/legacy-data-map.md`
- `docs/rbac-matrix.md`
- `docs/appcore-integration-surface.md`

O mapa deve distinguir quatro classes:

1. **PRESERVAR** — regra/fluxo funcional válido.
2. **NORMALIZAR** — comportamento válido com modelo inconsistente, como `zone/zona`, `valveTag/valve_tag`, `brand/location`.
3. **SUBSTITUIR** — detalhe de infraestrutura legado, como hook Supabase/localStorage fallback.
4. **DESCARTAR** — código/visual/template sem valor de produto.

## Regra de qualidade

Não escreva frases genéricas como “há um módulo de estoque”. Documente campos, operações e side effects concretos. Para cada regra importante, referencie o arquivo/função legado de onde ela foi inferida.

## Saída prática

Depois dos documentos, crie uma checklist de paridade funcional. Essa checklist será usada como gate para a migração: nenhuma função do legado pode desaparecer acidentalmente sem decisão explícita documentada.

Não refatore o legado nesta etapa.

# Prompt 09 — Relatórios, i18n e operação offline — CONCLUÍDO 100%

Finalize as capacidades transversais visíveis ao usuário.

## Relatórios

Preserve a semântica dos relatórios antigos:
- geral;
- por zona;
- válvulas críticas;
- manutenção recente.

O backend/application layer deve fornecer datasets/queries de relatório; a UI/PDF não deve recalcular regras críticas de forma paralela.

Gere PDF com identidade visual PROEXEL nova, tabelas legíveis, paginação, data/hora/locale e metadata suficiente. Evite cortar registros arbitrariamente sem informar; use paginação/continuação adequada.

## i18n

Migre PT, EN e ES para uma solução adequada ao Next.js. Extraia traduções existentes como referência, mas:
- normalize chaves;
- elimine strings importantes hardcoded;
- use locale para datas/números;
- tenha fallback controlado;
- não misture tradução com lógica de negócio.

## Offline/local-first

Defina UX explícita para:
- online;
- offline porém operando localmente;
- read-only/degraded;
- sync pending;
- sync error/conflict.

Não use uma simples bolinha `navigator.onLine` como verdade operacional. Se AppCore expõe runtime operational mode/health/sync state, use esses sinais.

Writes locais devem ter estado persistido e política de retry. Não finja sucesso remoto. Mostre “sincronização pendente” quando aplicável.

## Notifications

Reconstrua notificações úteis com base em dados reais: críticos, estoque abaixo do mínimo, OS relevantes e sync/health. Evite gerar ruído por eventos duplicados.

## PWA

Só preserve PWA se fizer sentido para o deployment. Se habilitar, defina cache strategy segura e não deixe service worker antigo do Vite controlar a nova aplicação por acidente.

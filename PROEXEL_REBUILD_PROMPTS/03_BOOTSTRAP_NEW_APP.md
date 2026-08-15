# Prompt 03 — Bootstrap do novo produto — CONCLUÍDO 100%

Crie a nova aplicação sem modificar o PROEXEL antigo.

## Frontend

Use o `admin-dashboard-templ` como ponto de partida real para `apps/web`.

Preserve o que é útil:
- Next.js/React/TypeScript;
- shadcn/base-ui já configurado;
- sidebar/layout/header;
- theme/preferences;
- tabelas/forms/dialogs;
- Biome e configuração de build.

Remova gradualmente conteúdo demo: CRM, finance, ecommerce, academy, legacy dashboards, links Github do template e dados de usuários falsos.

## Branding

Troque identificação do template por PROEXEL. Não replique o antigo “MaintPlant neon”. Crie uma interface profissional de manutenção industrial usando o design system existente, com densidade de informação adequada a desktop/tablet e boa operação em mobile.

## Backend/application host

Crie a composition root da aplicação consumindo AppCore por APIs públicas. Configure manifest, health, storage e serviços necessários sem alterar o Runtime.

## Configuração

Separe:
- configuração de aplicação;
- configuração de deployment;
- segredos;
- feature flags.

Forneça `.env.example` apenas para variáveis realmente necessárias e nunca copie credenciais do legado.

## Primeiro vertical slice

Entregue imediatamente um fluxo real mínimo:

1. backend inicia;
2. storage/migrations inicializam;
3. endpoint/command de health funciona;
4. UI carrega shell PROEXEL;
5. UI consegue fazer uma query real de lista de máquinas e categorias (mesmo com estado vazio);
6. estado vazio é bem desenhado, sem mocks permanentes.

Rode build/lint/tests.

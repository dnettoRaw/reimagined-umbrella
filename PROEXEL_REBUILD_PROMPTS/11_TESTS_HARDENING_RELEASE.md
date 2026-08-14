# Prompt 11 — Testes, hardening e release

Faça uma passagem final de qualidade como se o PROEXEL fosse entrar em produção.

## Testes

### Unit
- maintenance status em 150, 151, 180, 181 dias e nunca mantida;
- normalização de TAG/referências;
- estado OS;
- stock non-negative;
- permissions.

### Integration
- storage/migrations;
- RegisterMaintenance + stock + audit;
- create valve + stock ensure;
- auth/session;
- sync/outbox quando configurado;
- migration dry-run/idempotência.

### E2E UI
- login por papéis representativos;
- lista/filtro/detalhe de válvula;
- cadastro de válvula;
- manutenção completa com assinatura;
- OS;
- reposição;
- estoque;
- relatório.

## Segurança

- nenhuma secret no client bundle;
- password/hash seguro;
- authorization backend;
- upload de foto validado;
- input validation;
- proteção contra IDOR;
- logs/audit sem segredo;
- endpoints administrativos protegidos;
- headers/cookies apropriados;
- dependency audit conforme tooling do projeto.

## Resiliência

Teste:
- backend/runtime indisponível;
- storage cheio/erro;
- retry de command;
- queda de rede;
- estado read-only/degraded;
- sync atrasado;
- refresh durante operação.

## Performance

Meça operações relevantes com volume plausível de válvulas/histórico. Evite N+1 no dashboard e detalhe. Paginar audit/history. Não carregar todas as fotos em resolução completa na lista.

## Cleanup

Remova:
- páginas demo do admin template;
- dados falsos;
- imports mortos;
- compat shims temporários que já não são usados;
- qualquer dependência Supabase/Vite do novo produto se não for parte deliberada do design final;
- credenciais/test passwords.

## Documentação final

Atualize:
- README
- arquitetura
- RBAC
- backup/restore
- migration runbook
- deployment
- troubleshooting
- release notes

## Gate final

Rode todos os comandos disponíveis: format/check, lint, typecheck, unit/integration/e2e relevantes, build Rust e build Next. Se algum falhar, corrija antes de declarar conclusão. Liste apenas pendências reais que não podem ser resolvidas com o código presente.

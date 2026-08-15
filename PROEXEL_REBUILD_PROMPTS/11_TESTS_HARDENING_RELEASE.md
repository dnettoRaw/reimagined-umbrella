# Prompt 11 - Testes, hardening e release - CONCLUÍDO 100%

## Cobertura concluída

- criação de categoria, máquina e componentes;
- complexidade padrão e override;
- bloqueio de operador incompatível;
- OS com seleção congelada e adição posterior sem expansão;
- execução obrigatória do guia e atualização de status;
- foto de guia ligada, congelada e protegida contra remoção;
- substituição com identidade física anterior preservada;
- migração canônica e importação legada idempotente;
- persistência durável, concorrência e falha sem commit;
- RBAC, usuários, hashes redigidos e último admin;
- paginação de volume e datasets de relatório;
- boundary de dependências.

## E2E concluído

O Playwright inicia um stack isolado, cria categoria/máquina/componente/OS,
executa o guia com técnico elegível, conclui a ordem, registra e aprova reposição,
exporta PDF, administra usuário com nível e PIN e verifica viewport móvel.

## Gates concluídos

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
npm run check
npx tsc --noEmit
npm run build
./proexel/scripts/e2e.sh
npm audit --audit-level=high
```

Todos passaram em 2026-08-15. Os procedimentos específicos de backup, restore,
cutover e troubleshooting estão fechados em `docs/`.

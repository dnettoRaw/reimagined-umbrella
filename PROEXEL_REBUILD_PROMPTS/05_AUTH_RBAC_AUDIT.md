# Prompt 05 — Auth, RBAC e auditoria

Substitua o login/RBAC legado por uma implementação compatível com a arquitetura nova e com AppCore.

## Antes de implementar

Extraia a matriz real de permissões do PROEXEL antigo. O legado contém pelo menos:

- admin
- chefe
- compras
- tecnico

Há views diferentes por papel e ações adicionais dentro das views. Não deduza autorização apenas da lista de páginas.

## Requisitos

- Autenticação server/application-side; não confiar em hash de senha executado apenas no browser.
- Sessão com expiração explícita e renovação segura quando aplicável.
- Password storage usando algoritmo apropriado e parâmetros versionados; se migrar hashes antigos, trate como migração temporária e retire fallback de senha plaintext.
- Permissão no command/query handler, não só em componente React.
- UI deve refletir permissões para UX, mas backend é autoridade.

Crie permissions granulares, por exemplo:
- valve.read / valve.create / valve.update
- maintenance.read / maintenance.execute
- order.read / order.create / order.status.change / order.delete
- stock.read / stock.adjust
- restock.create / restock.review / restock.delete
- supplier.manage
- audit.read
- report.generate
- admin.manage

Mapeie roles → permissions em configuração/testes.

## Auditoria automática

Implemente middleware/decorator/application service que produza audit events em writes importantes. Não dependa de chamadas manuais como `logAction(...)` em cada tela.

Inclua:
- actor id/nome
- role(s)
- command/action
- aggregate/entity
- entity id
- before/after redigidos quando necessário
- description
- timestamp
- correlation/trace id
- result success/failure quando adequado

Dados sensíveis e credenciais nunca devem aparecer no audit.

## Testes

Teste que um técnico não consegue executar comandos administrativos mesmo chamando a API diretamente. Teste cada role principal e operações críticas.

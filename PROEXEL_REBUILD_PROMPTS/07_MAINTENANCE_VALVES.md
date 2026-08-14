# Prompt 07 — Vertical slice Válvulas + Manutenção

Complete ponta a ponta o núcleo do produto.

## Válvulas

Implemente CRUD real com regras de autorização e auditoria. Preserve todos os campos relevantes do legado. Normalize TAG em uma policy única. Defina claramente se alteração de TAG é permitida; fotos e históricos devem continuar ligados por `valve_id`.

Ao criar uma válvula com referência de kit:
- normalize a referência;
- crie/garanta o StockItem correspondente de forma idempotente quando essa regra estiver confirmada no legado;
- quantidade inicial 0 e mínimo inicial coerente com a regra antiga/configuração;
- não sobrescreva dados existentes de estoque.

## Fotos

Substitua bucket/fallback/localStorage por attachment storage apropriado. Valide content type, tamanho e dimensões razoáveis. Gere thumbnail se necessário. Não armazene data URL gigante em tabelas operacionais se a infraestrutura suportar blob/file storage.

## Status

A policy de 150/180 dias deve ser a única fonte de verdade. UI recebe status calculado/contratado sem reimplementar datas de forma divergente.

## Manutenção

Fluxo:
1. abrir válvula;
2. iniciar manutenção;
3. selecionar tipo;
4. preencher serviço/notas;
5. indicar troca de kit;
6. assinatura;
7. revisar;
8. confirmar;
9. operação backend transacional/idempotente;
10. atualizar status/timeline/estoque sem reload manual.

Se kit_changed=true e a válvula não possui kit/ref compatível, mostre tratamento explícito; não falhe silenciosamente.

Se o estoque estiver em zero, não permita ficar negativo. Defina UX clara para “manutenção registrada, consumo de estoque pendente” versus bloquear operação conforme regra mais coerente e documente a decisão. Prefira preservar registro da manutenção física sem falsificar saldo.

## Histórico

Timeline por válvula e histórico global pesquisável. Mostrar técnico, data, tipo, troca de kit, serviço e notas conforme permissão. Assinatura deve ser visualizável apenas onde apropriado.

## Testes e2e

Crie pelo menos um cenário completo: cadastrar válvula → garantir kit → executar manutenção → estoque decrementa uma vez → status/data atualizam → audit event aparece.

# Prompt 08 — OS, estoque, reposição e fornecedores — CONCLUÍDO 100%

Implemente os módulos operacionais restantes sem reutilizar hooks Supabase antigos.

## Service Orders

Preserve campos do legado e normalize estados:
- `aberta` legado deve ser tratado de forma compatível com o estado canônico escolhido;
- pendente/open;
- in_progress/andamento;
- completed/concluida.

Defina máquina de estados simples e teste transições permitidas. Não permita qualquer string arbitrária.

Criação é permitida apenas aos papéis confirmados pelo mapa RBAC. Mudança de status e exclusão também obedecem permission handlers.

## Schedule

A antiga Agenda calculava prioridade por válvulas críticas. A implementação nova preserva a informação útil por meio de componentes críticos e do estado determinístico das máquinas, sem reproduzir a modelagem antiga.

## Restock Requests

Fluxo de técnico: informar nome/ator real da sessão sempre que possível, referência e descrição do problema. Se o legado exige nome manual por razões operacionais, preserve apenas se houver necessidade real documentada.

Fluxo de chefe/admin/compras: visualizar. Revisão approve/reject deve registrar revisor e timestamp. Exclusão deve ser auditada.

## Stock

- referência normalizada e unique;
- quantidade nunca negativa;
- mínimo configurável;
- separar manufacturer de location;
- ajuste de quantidade com reason obrigatório para ajustes manuais relevantes;
- histórico de movimentos recomendado em vez de apenas sobrescrever saldo.

Implemente `stock_movements` se isso melhorar rastreabilidade: receipt, consumption, correction, migration. O saldo pode ser derivado ou materializado de forma consistente.

## Suppliers

CRUD com nome e contato obrigatórios conforme legado. Validar email/url quando presentes. Auditoria e RBAC.

## Compras

Reconstrua a tela `Compras` a partir do comportamento real encontrado: estoque, mínimo, reposições e fornecedores. Não preserve inconsistências de nomenclatura do legado.

## Testes

- transições OS;
- aprovação/rejeição reposição;
- ajuste de estoque sem negativo;
- concorrência/retry em consumo;
- CRUD fornecedor com autorização.

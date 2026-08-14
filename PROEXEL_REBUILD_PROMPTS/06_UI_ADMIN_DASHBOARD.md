# Prompt 06 — Reconstrução da UI sobre admin-dashboard

Agora substitua o conteúdo demo do admin-dashboard por uma UI PROEXEL completa.

## Navegação

Configure sidebar baseada em permissão, com itens reais:

- Overview
- Valves
- Maintenance
- Service Orders / Schedule
- Stock
- Purchasing / Restock
- Suppliers
- History / Audit
- Reports
- Administration (se necessário)

Use Lucide icons existentes. Remova páginas demo e grupos Legacy/Misc que não pertencem ao produto.

## Overview

Crie dashboard operacional com dados reais:
- total de válvulas;
- OK / warning / critical;
- percentual em dia;
- OS abertas/em andamento/concluídas;
- itens abaixo do mínimo;
- manutenção recente;
- zonas com maior criticidade;
- próximos trabalhos agendados.

Cards devem ser clicáveis quando fizer sentido e levar a filtros correspondentes.

## Valves

Tabela TanStack com:
- TAG
- zona
- fabricante
- tipo/DN
- kit
- última manutenção
- status
- ações conforme permissão

Incluir search, filtros por zona/status/tipo, sorting, paginação/virtualização conforme volume, estados vazios e loading.

Detalhe da válvula em página ou sheet bem estruturado, com dados técnicos, foto, timeline de manutenção, estoque do kit e ações.

Cadastro/edição em formulário tipado com React Hook Form + Zod ou stack já presente no template.

## Maintenance

Tela dedicada para:
- executar manutenção;
- visualizar procedimento/guia se houver lógica correspondente no legado;
- selecionar preventiva/corretiva;
- serviço/notas;
- confirmar troca de kit;
- capturar assinatura responsiva para mouse/touch;
- mostrar impacto no estoque antes de finalizar quando possível.

## Orders/Schedule

Tabela/kanban/calendário conforme fizer sentido, sem criar três experiências redundantes. Preservar estados e permissões reais.

## Stock/Purchasing/Suppliers

Criar telas densas, claras e operacionais, com indicadores de quantidade mínima, ajustes controlados, sugestões de reposição e gestão de fornecedores.

## Histórico/Audit

Filtros por ator, ação, entidade e período; paginação; detail viewer para before/after quando permitido.

## UX

- responsivo;
- navegação por teclado onde os componentes suportarem;
- feedback Sonner/toasts;
- dialogs de confirmação apenas para ações realmente destrutivas;
- skeletons em vez de layout pulando;
- erros acionáveis;
- sem emoji como ícone principal da interface;
- sem cores hardcoded se o design system já oferece tokens.

Não copie HTML/SCSS do PROEXEL antigo.

# MASTER PROMPT - Evolução completa do PROEXEL - CONCLUÍDO 100%

## Objetivo entregue

O PROEXEL foi evoluído sobre a arquitetura existente, sem rewrite paralelo, para
representar máquinas compostas por componentes arbitrários e conduzir operadores
em inspeções estruturadas com autorização técnica e rastreabilidade.

```text
Machine
  -> MachineItem[]
       -> ItemCategory
            -> MaintenanceGuide
       -> InstalledComponent
       -> ReplacementSpecification
       -> history

ServiceOrder
  -> ServiceOrderTask[]
       -> immutable snapshots
       -> ItemInspection
```

## Requisitos concluídos

- Domínio PROEXEL isolado do runtime AppCore genérico.
- Categorias dinâmicas criadas pela UI, com campos tipados e peças recomendadas.
- Editor amigável de guias estruturados com nove tipos de etapa e ordenação.
- Máquinas com componentes, fotos, estado derivado e histórico.
- Posição funcional separada da unidade física substituível.
- Complexidade e nível máximo do operador validados em 1..5 no backend.
- OS para um, alguns ou todos os componentes com snapshots imutáveis.
- Execução passo a passo, resultados estruturados, findings e evidências.
- Fotos de máquina, componente, guia, inspeção e substituição.
- Usuários, papéis, senha, PIN, ativação, nível técnico e auditoria.
- Estoque, compras, fornecedores, relatórios, PDF e notificações preservados.
- PT, EN, ES e FR obrigatórios com paridade tipada.
- Migração legada determinística e migração automática do estado canônico.
- Remoção de rotas, DTOs, commands, queries, UI e catálogo Valve do runtime.

## Regras críticas concluídas

- `ComplexityLevel` e `maximum_repair_level` aceitam somente 1..5.
- Operador incompatível não assume, inicia ou conclui tarefa.
- Categoria inativa não recebe novos itens.
- Item deve pertencer à máquina da OS.
- Etapas obrigatórias e fotos exigidas são validadas.
- Unidade de medição deve corresponder ao guia congelado.
- OS não fecha com tarefas pendentes.
- Fotos congeladas em snapshot não podem ser removidas.
- Estado da máquina deriva deterministicamente dos componentes ativos.

## Qualidade concluída

- 29 testes Rust.
- Clippy em todos os targets com warnings negados.
- Biome e TypeScript.
- Build de produção Next.js.
- Playwright em stack AppCore isolado.
- npm audit sem vulnerabilidades altas.

Os documentos operacionais em `docs/` são a referência atual. Os mapas de
legado mantêm termos antigos apenas para explicar e executar a importação.

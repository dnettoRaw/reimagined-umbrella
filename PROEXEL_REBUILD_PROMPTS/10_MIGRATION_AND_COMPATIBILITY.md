# Prompt 10 - Migração e compatibilidade - CONCLUÍDO 100%

## Importador legado

O crate `proexel-migration` aceita o contrato antigo, aliases portugueses e
referências de fotos. A importação:

- cria uma ItemCategory Valve reutilizável;
- cria máquinas determinísticas por zona;
- converte cada registro antigo em MachineItem;
- converte manutenção em ItemInspection;
- converte OS para tarefas com snapshots;
- preserva estoque, reposições, fornecedores, auditoria e metadados de fotos;
- gera IDs estáveis, checksum, warnings e relatórios JSON/Markdown;
- suporta dry-run e repetição idempotente por batch.

Os nomes antigos existem somente nos DTOs/mapas de importação. Não há shim de
runtime nem duas entidades representando o mesmo componente.

## Migração canônica

Ao abrir um estado schema v1, a aplicação cria `.schema-v1.json.bak`, converte
os registros operacionais para schema v2 e persiste a nova forma. Estados v2 são
lidos diretamente. Ambos os caminhos têm testes.

O cutover de produção usa `docs/migration-runbook.md`; a execução depende do
export e dos binários reais fornecidos pelo operador, não de código adicional.

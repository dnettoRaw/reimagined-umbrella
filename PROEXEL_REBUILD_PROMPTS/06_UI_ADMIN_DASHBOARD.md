# Prompt 06 - UI administrativa - CONCLUÍDO 100%

## Rotas entregues

- Visão geral.
- Máquinas e detalhe com componentes/fotos/histórico.
- Categorias de componentes e editor do guia.
- Ordens de serviço e atribuição de operadores.
- Minhas ordens e execução guiada.
- Estoque, compras e fornecedores.
- Relatórios/PDF, notificações e auditoria.
- Usuários, papéis, nível técnico, senha, PIN e ativação.
- Configurações e idioma.

## Editor de categoria

O administrador gerencia propriedades, campos personalizados tipados, peças
recomendadas e passos estruturados. Passos podem ser adicionados, removidos e
reordenados; choice, ranges, unidade, obrigatoriedade, avisos e fotos de
referência são editáveis sem JSON manual.

## Ergonomia e segurança

A UI usa os componentes existentes do admin dashboard, ícones Lucide, estados
vazios, tabelas, filtros, dialogs e layout responsivo. Botões refletem RBAC, mas
o serviço repete todas as autorizações e regras de domínio.

Todo texto de produto possui tradução PT/EN/ES/FR. O E2E verifica o fluxo em
desktop e ausência de overflow no viewport móvel.

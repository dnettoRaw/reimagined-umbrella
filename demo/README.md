# PROEXEL demo

A demonstração usa a aplicação canônica em `proexel/apps/web`; não existe uma
segunda implementação de páginas ou componentes. O modo demo substitui somente
sessão e transporte de dados, mantendo o layout, as rotas, os formulários, as
permissões e os fluxos operacionais reais.

Os dados iniciais e as alterações de cada visitante ficam no `localStorage` do
próprio navegador. Senhas e PINs digitados nos formulários de demonstração são
descartados e nunca são armazenados.

## Vercel

Ao importar o repositório, configure **Root Directory** como
`proexel/apps/web`. O `vercel.json` dessa pasta executa `npm ci` e
`npm run build:demo` automaticamente. Não são necessárias variáveis de
ambiente, banco de dados, AppCore ou acesso ao submódulo.

## Local

```bash
cd proexel/apps/web
npm ci
npm run build:demo
PROEXEL_DEMO=1 NEXT_PUBLIC_PROEXEL_DEMO=1 npm start -- --port 3030
```

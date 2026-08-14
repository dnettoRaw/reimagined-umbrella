import { Factory } from "lucide-react";

import { LoginForm } from "../_components/login-form";

export default async function LoginPage({ searchParams }: { readonly searchParams: Promise<{ next?: string }> }) {
  const { next } = await searchParams;
  return (
    <main className="grid min-h-dvh bg-background lg:grid-cols-[minmax(320px,0.75fr)_1.25fr]">
      <section className="flex flex-col justify-between border-b bg-foreground p-8 text-background lg:border-r lg:border-b-0 lg:p-12">
        <div className="flex items-center gap-3">
          <Factory className="size-7" />
          <strong className="text-xl">PROEXEL</strong>
        </div>
        <div className="mt-16 max-w-md lg:mt-0">
          <h1 className="font-heading font-semibold text-3xl sm:text-4xl">Operação industrial sob controle.</h1>
          <p className="mt-4 text-background/70">Válvulas, manutenção, ordens e estoque em uma instalação local.</p>
        </div>
        <p className="mt-12 text-background/60 text-xs">Acesso restrito a operadores autorizados.</p>
      </section>
      <section className="flex items-center justify-center p-6 sm:p-10">
        <div className="w-full max-w-sm">
          <div className="mb-8">
            <h2 className="font-heading font-semibold text-2xl">Iniciar sessão</h2>
            <p className="mt-2 text-muted-foreground text-sm">Use as credenciais atribuídas à sua função.</p>
          </div>
          <LoginForm next={next} />
        </div>
      </section>
    </main>
  );
}

# Prompt 07 - Máquinas, componentes e manutenção guiada - CONCLUÍDO 100%

Esta fase substitui integralmente o antigo vertical slice centrado em válvulas.

## Entrega

1. Administrador cria uma ItemCategory reutilizável.
2. O guia da categoria define passos estruturados e fotos de referência.
3. Chefe/admin cria uma Machine e adiciona MachineItems.
4. Cada item herda ou sobrescreve complexidade 1..5.
5. A unidade física instalada mantém fabricante, modelo, part number, serial e specs.
6. A especificação de substituição mantém compatibilidade e peças equivalentes.
7. A OS congela máquina, itens, categoria, guia, fotos e complexidade.
8. Operador elegível executa passo a passo e registra resultados/fotos/findings.
9. A inspeção atualiza o item e o estado derivado da máquina.
10. Substituições preservam o histórico completo da posição funcional.

Uma válvula pneumática agora é apenas uma categoria possível criada pela UI. O
runtime não possui entidade Valve, rotas especializadas ou procedimento paralelo.

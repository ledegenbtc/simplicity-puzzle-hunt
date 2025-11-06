#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║        DEMONSTRAÇÃO: CADA PUZZLE GERA UM CMR ÚNICO           ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo

# Test 1: Mesmo secret gera mesmo CMR
echo "═══ TESTE 1: Mesmo secret → Mesmo CMR ═══"
echo
echo "Compilando 'hello' três vezes..."

CMR1=$(cargo run --bin export-program -- "hello" 2>/dev/null | \
       hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
       jq -r '.cmr')
CMR2=$(cargo run --bin export-program -- "hello" 2>/dev/null | \
       hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
       jq -r '.cmr')
CMR3=$(cargo run --bin export-program -- "hello" 2>/dev/null | \
       hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
       jq -r '.cmr')

echo "CMR #1: $CMR1"
echo "CMR #2: $CMR2"
echo "CMR #3: $CMR3"
echo

if [ "$CMR1" = "$CMR2" ] && [ "$CMR2" = "$CMR3" ]; then
    echo "✅ SUCESSO: Todos idênticos! (determinístico)"
else
    echo "❌ ERRO: CMRs deveriam ser idênticos!"
fi

echo
echo "─────────────────────────────────────────────────────────────────"
echo

# Test 2: Secrets diferentes geram CMRs diferentes
echo "═══ TESTE 2: Secrets diferentes → CMRs diferentes ═══"
echo
echo "Compilando 3 secrets diferentes..."
echo

declare -A CMRS

SECRETS=("hello" "world" "bitcoin")

for secret in "${SECRETS[@]}"; do
    CMR=$(cargo run --bin export-program -- "$secret" 2>/dev/null | \
          hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
          jq -r '.cmr')
    CMRS[$secret]=$CMR
    echo "Secret: '$secret'"
    echo "  CMR: $CMR"
    echo
done

echo "─────────────────────────────────────────────────────────────────"
echo

# Verificar que são todos diferentes
echo "Verificando que todos os CMRs são DIFERENTES..."
echo

if [ "${CMRS[hello]}" != "${CMRS[world]}" ] && \
   [ "${CMRS[hello]}" != "${CMRS[bitcoin]}" ] && \
   [ "${CMRS[world]}" != "${CMRS[bitcoin]}" ]; then
    echo "✅ SUCESSO: Todos os CMRs são únicos!"
else
    echo "❌ ERRO: Algum CMR está duplicado!"
fi

echo
echo "─────────────────────────────────────────────────────────────────"
echo

# Test 3: Mostrar a diferença no bytecode
echo "═══ TESTE 3: Diferença no bytecode ═══"
echo
echo "Pegando o bytecode decodificado para 'hello' e 'world'..."
echo

DECODE_HELLO=$(cargo run --bin export-program -- "hello" 2>/dev/null | \
               hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
               jq -r '.commit_decode' | grep -o 'const 0x[0-9a-f]*')

DECODE_WORLD=$(cargo run --bin export-program -- "world" 2>/dev/null | \
               hal-simplicity simplicity simplicity info "$(cat)" 2>/dev/null | \
               jq -r '.commit_decode' | grep -o 'const 0x[0-9a-f]*')

echo "'hello':"
echo "  $DECODE_HELLO"
echo
echo "'world':"
echo "  $DECODE_WORLD"
echo

echo "A constante TARGET_HASH é diferente!"
echo "Por isso o CMR (hash do bytecode) também é diferente."

echo
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                        CONCLUSÃO                              ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo
echo "✅ Mesmo secret  → Mesmo CMR (determinístico)"
echo "✅ Secret diferente → CMR diferente (único)"
echo
echo "Cada puzzle é um contrato ÚNICO com:"
echo "  • CMR único (impressão digital)"
echo "  • Endereço Bitcoin único"
echo "  • Bytecode único"
echo

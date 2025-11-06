#!/bin/bash

echo "🎯 PUZZLES DISPONÍVEIS"
echo "======================="
echo ""

# Contador de puzzles
count=0

# Busca todos os arquivos de puzzle
for puzzle_file in puzzle_*.json; do
    if [ -f "$puzzle_file" ]; then
        count=$((count + 1))

        # Extrai informações do arquivo JSON
        address=$(jq -r '.address' "$puzzle_file")
        amount=$(jq -r '.amount' "$puzzle_file")
        hash=$(jq -r '.hash' "$puzzle_file")
        hint=$(jq -r '.hint' "$puzzle_file")

        echo "📦 Puzzle #$count"
        echo "   Arquivo: $puzzle_file"
        echo "   📍 Endereço: $address"
        echo "   💰 Prêmio: $amount L-BTC"
        echo "   🔐 Hash: $hash"
        echo "   💡 Dica: $hint"

        # Verifica se há UTXOs no endereço (se elements-cli estiver disponível)
        if [ -x "./elements-cli" ]; then
            echo -n "   📊 Status: "
            # Use scantxoutset para encontrar UTXOs de qualquer endereço
            scan_result=$(./elements-cli -chain=liquidtestnet scantxoutset start "[\"addr($address)\"]" 2>/dev/null)

            if [ ! -z "$scan_result" ]; then
                # Extrai o total do resultado
                total=$(echo "$scan_result" | jq -r '.total_unblinded_bitcoin_amount // 0')
                unspent_count=$(echo "$scan_result" | jq -r '.unspents | length')

                if [ "$unspent_count" != "0" ] && [ "$unspent_count" != "null" ] && [ "$total" != "0" ]; then
                    echo "ATIVO (Saldo: $total L-BTC, $unspent_count UTXO(s))"
                else
                    echo "RESOLVIDO ou SEM FUNDOS"
                fi
            else
                echo "ERRO ao verificar"
            fi
        fi

        echo ""
    fi
done

if [ $count -eq 0 ]; then
    echo "❌ Nenhum puzzle encontrado."
    echo ""
    echo "💡 Para criar um puzzle, use:"
    echo "   cargo run --bin create-puzzle -- \"senha\" 0.1"
else
    echo "========================"
    echo "📊 Total de puzzles: $count"
fi
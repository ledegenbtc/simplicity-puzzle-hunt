# 📚 Guia de Comandos - Bitcoin Puzzle Hunt

## 📋 Listar Puzzles Disponíveis

### Opção 1: Script Bash
```bash
./list-puzzles.sh
```

### Opção 2: Comando Rust
```bash
cargo run --bin list-puzzles
```

## 🎯 Criar Novo Puzzle

### Sintaxe
```bash
cargo run --bin create-puzzle -- <secret> <amount> [custom_hint]
```

### Exemplos

#### Com dica padrão (número de caracteres)
```bash
cargo run --bin create-puzzle -- "satoshi" 0.1
```
Dica gerada: "The password has 7 characters"

#### Com dica personalizada
```bash
# Dicas educativas
cargo run --bin create-puzzle -- "bitcoin" 0.5 "Nome da primeira criptomoeda"
cargo run --bin create-puzzle -- "nakamoto" 0.3 "Sobrenome do criador do Bitcoin"

# Dicas divertidas
cargo run --bin create-puzzle -- "moon" 0.2 "Destino favorito dos HODLers 🚀"
cargo run --bin create-puzzle -- "hodl" 0.15 "Erro de digitação que virou filosofia"
cargo run --bin create-puzzle -- "pizza" 0.1 "22 de maio é o dia da ___ do Bitcoin"

# Dicas técnicas
cargo run --bin create-puzzle -- "hash" 0.2 "Função criptográfica usada no Bitcoin"
cargo run --bin create-puzzle -- "block" 0.3 "Unidade de dados na blockchain"
cargo run --bin create-puzzle -- "wallet" 0.25 "Onde você guarda suas chaves privadas"

# Dicas enigmáticas
cargo run --bin create-puzzle -- "2009" 0.4 "O ano em que tudo começou"
cargo run --bin create-puzzle -- "genesis" 0.35 "O primeiro de todos"
cargo run --bin create-puzzle -- "21M" 0.5 "O limite máximo"
```

## 💰 Adicionar Fundos ao Prêmio

```bash
cargo run --bin add-to-pot -- puzzle_<hash>.json 0.05
```

Exemplo:
```bash
cargo run --bin add-to-pot -- puzzle_7cadab45.json 0.05
```

## 🏆 Resolver Puzzle

```bash
# Primeiro, obter um endereço para receber o prêmio
./elements-cli getnewaddress

# Depois, resolver o puzzle
cargo run --bin solve-puzzle -- puzzle_<hash>.json "secret" <your_address>
```

Exemplo:
```bash
cargo run --bin solve-puzzle -- puzzle_7cadab45.json "lucas" tex1q...
```

## 🔍 Verificar Status de um Puzzle

```bash
# Ver UTXOs de um endereço específico
./elements-cli listunspent 0 9999999 '["<puzzle_address>"]'
```

## 📤 Exportar Programa Simplicity

```bash
cargo run --bin export-program -- puzzle_<hash>.json
```

## 💡 Dicas para Criar Bons Puzzles

### Níveis de Dificuldade

**Fácil** (0.01-0.05 L-BTC)
- Palavras conhecidas do universo crypto
- Dicas diretas e claras
- Exemplos: "bitcoin", "satoshi", "moon", "hodl"

**Médio** (0.05-0.2 L-BTC)
- Requer algum conhecimento específico
- Dicas com duplo sentido
- Exemplos: "genesis", "halving", "lightning"

**Difícil** (0.2+ L-BTC)
- Palavras menos óbvias ou combinações
- Dicas enigmáticas
- Exemplos: hashes específicos, datas importantes, referências históricas

### Exemplos de Dicas Criativas

```bash
# Dica com contexto histórico
cargo run --bin create-puzzle -- "pizzaday" 0.3 "10.000 BTC por duas pizzas (uma palavra)"

# Dica com referência cultural
cargo run --bin create-puzzle -- "lambo" 0.2 "Carro dos sonhos de todo crypto trader"

# Dica matemática
cargo run --bin create-puzzle -- "2140" 0.4 "Ano em que o último Bitcoin será minerado"

# Dica com emoji
cargo run --bin create-puzzle -- "whale" 0.5 "🐋 Grande detentor de Bitcoin"

# Dica em forma de charada
cargo run --bin create-puzzle -- "private" 0.3 "Tipo de chave que você nunca deve compartilhar"
```

## 🛠 Solução de Problemas

### Puzzle não aparece na lista
- Verifique se o arquivo JSON existe no diretório
- Execute `ls puzzle_*.json` para ver todos os puzzles

### Erro ao criar puzzle
- Certifique-se que o elementsd está rodando
- Verifique se tem fundos na carteira: `./elements-cli getbalance`
- Use aspas para secrets com espaços: `"minha senha"`

### Erro ao resolver puzzle
- Confirme que o secret está correto
- Verifique se o puzzle ainda tem fundos
- Certifique-se de ter editado o arquivo solve_puzzle.rs com as informações do UTXO

## 📊 Estatísticas e Análise

Para ver informações detalhadas sobre todos os puzzles:
```bash
# Lista completa com status
./list-puzzles.sh

# Apenas arquivos JSON
ls -la puzzle_*.json

# Ver conteúdo de um puzzle específico
cat puzzle_<hash>.json | jq '.'
```

## 🔍 Verificação e Debug

### Verificar Compilação do Contrato
```bash
# Verificar se o contrato foi criado corretamente
cargo run --bin check-contract -- <hash> [address]

# Exemplo
cargo run --bin check-contract -- 0x20f0e912902bfdc1ea47cdb5eadc6f5c1b3453f406f38dd34a92d0b30a270e22

# Verificar e validar endereço
cargo run --bin check-contract -- 0x20f0e912... tex1p6k8njks70y4xkv...
```

**O que mostra:**
- CMR (Commitment Merkle Root) do contrato
- Estrutura Taproot (internal key, merkle root)
- Endereço gerado
- Validação se endereço está correto

### Verificar Status do Puzzle
```bash
# Por endereço
cargo run --bin verify-puzzle -- tex1p6k8njks70y4xkv...

# Por TXID da transação
cargo run --bin verify-puzzle -- e7f815d4013f10b8294369c3fff126aef497...
```

**O que mostra:**
- Se o puzzle ainda está ativo ou foi resolvido
- Detalhes do UTXO (se ativo)
- Confirmações da transação
- Se é uma transação de solução Simplicity

**Exemplo - Puzzle Ativo:**
```
✅ PUZZLE IS ACTIVE
   TXID: e7f815d4...
   Amount: 0.0001 L-BTC
   Confirmations: 5
```

**Exemplo - Puzzle Resolvido:**
```
❌ PUZZLE WAS SOLVED OR NEVER FUNDED
   No UTXO found at this address
```

## 🎮 Organizando uma Caça ao Tesouro

1. **Prepare múltiplos puzzles** com dificuldades variadas
2. **Publique as dicas** gradualmente (redes sociais, Discord, etc)
3. **Aumente os prêmios** conforme o tempo passa sem solução
4. **Monitore** os puzzles com o comando list-puzzles
5. **Anuncie** quando alguém resolver um puzzle

---

**Divirta-se criando e resolvendo puzzles!** 🎯
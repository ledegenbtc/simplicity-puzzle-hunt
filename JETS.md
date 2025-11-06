# 🚀 Jets e Verificação de Contratos Simplicity

## 📚 O que são Jets?

**Jets** são operações primitivas otimizadas do Simplicity. São como "funções built-in" de baixo nível que executam operações específicas de forma eficiente e segura.

### Por que Jets?

Simplicity é uma linguagem de baixo nível muito expressiva, mas pode ser verbosa. Jets permitem:
- ✅ **Performance**: Operações comuns são otimizadas
- ✅ **Segurança**: Implementações auditadas e testadas
- ✅ **Eficiência**: Reduz tamanho de programas e custo de execução

## 🔧 Jets Usados no Puzzle Hunt

No contrato `puzzle_jackpot.simf`, usamos estes jets:

### 1. `jet::sha_256_ctx_8_init()`
**O que faz:** Inicializa um contexto SHA256 para hashing
```rust
let hasher: Ctx8 = jet::sha_256_ctx_8_init();
```
- **Tipo de retorno:** `Ctx8` (contexto SHA256)
- **Uso:** Ponto de partida para calcular hash SHA256

### 2. `jet::sha_256_ctx_8_add_32(hasher, data)`
**O que faz:** Adiciona 32 bytes de dados ao contexto SHA256
```rust
let hasher: Ctx8 = jet::sha_256_ctx_8_add_32(hasher, preimage);
```
- **Parâmetros:**
  - `hasher`: Contexto SHA256
  - `data`: u256 (32 bytes) a serem adicionados ao hash
- **Tipo de retorno:** `Ctx8` (contexto atualizado)

### 3. `jet::sha_256_ctx_8_finalize(hasher)`
**O que faz:** Finaliza o cálculo e retorna o hash SHA256
```rust
let hash: u256 = jet::sha_256_ctx_8_finalize(hasher);
```
- **Parâmetro:** Contexto SHA256
- **Tipo de retorno:** `u256` (hash de 32 bytes)

### 4. `jet::eq_256(a, b)`
**O que faz:** Compara dois valores de 256 bits (32 bytes)
```rust
assert!(jet::eq_256(computed_hash, target_hash));
```
- **Parâmetros:** Dois valores `u256`
- **Tipo de retorno:** `bool` (true se iguais, false caso contrário)
- **Nota:** Quando esta operação falha dentro de um `assert!`, você vê o erro:
  ```
  error code: -26
  non-mandatory-script-verify-flag (Assertion failed inside jet)
  ```

## 🔍 Entendendo Erros de Jets

### Erro: "Assertion failed inside jet"
```
error code: -26
error message:
non-mandatory-script-verify-flag (Assertion failed inside jet)
```

**O que significa:**
- O contrato Simplicity foi executado
- Um jet retornou um valor que falhou em uma assertion
- No caso do puzzle, significa: **O secret está incorreto!**

**Causa no Puzzle Hunt:**
```rust
let computed_hash = sha2(secret);  // Calcula hash do secret fornecido
assert!(jet::eq_256(computed_hash, target_hash));  // Compara com hash esperado
```

Se `computed_hash != target_hash`, o `jet::eq_256` retorna `false` e a assertion falha.

**Como resolver:**
1. Verifique se o secret está correto
2. Use o comando `solve-puzzle` que verifica o hash antes de tentar gastar
3. Confira se está usando o formato correto do secret

## 🛠️ Novas Ferramentas de Verificação

### 1. `check-contract` - Verificar Compilação do Contrato

**Uso:**
```bash
cargo run --bin check-contract -- <hash> [address]
```

**Exemplos:**
```bash
# Apenas compilar e gerar endereço
cargo run --bin check-contract -- 0x20f0e912902bfdc1ea47cdb5eadc6f5c1b3453f406f38dd34a92d0b30a270e22

# Verificar se endereço está correto
cargo run --bin check-contract -- 0x20f0e912... tex1p6k8njks70y4xkv...
```

**O que faz:**
1. ✅ Compila o contrato Simplicity com o hash fornecido
2. ✅ Mostra o **CMR (Commitment Merkle Root)**
3. ✅ Gera o endereço Taproot
4. ✅ Verifica se o endereço gerado bate com o esperado

**Output exemplo:**
```
🔧 CONTRACT VERIFICATION
=========================

📝 Input:
   Target Hash: 0x20f0e912902bfdc1ea47cdb5eadc6f5c1b3453f406f38dd34a92d0b30a270e22

⚙️  Compiling Simplicity contract...
✅ Contract compiled successfully!

🔐 Contract Details:
   CMR (Commitment Merkle Root):
   0xb47a817688a2e95f0bf03118c96f9325b1d9bf13ae2739a66ed8a59aa1016d66
   Length: 32 bytes

🏗️  Building Taproot Address...
✅ Taproot Structure:
   Internal Key: 50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0
   Merkle Root:  279a39548d064f901cd7d8fa88126347e9b9b68458712c9fd316ae342126c439

📍 Generated Address:
   tex1p6k8njks70y4xkv25n2w8vxvapjt8qhx7uen0ncrzr974m6g7xyns78dq6d

🔍 Verification:
   ✅ SUCCESS! Address matches expected address!

📊 Summary:
   Contract compilation: ✅ OK
   CMR generation:       ✅ OK
   Address generation:   ✅ OK
   Address verification: ✅ MATCH
```

**Casos de uso:**
- 🔍 Verificar se um puzzle foi criado corretamente
- 🛡️ Auditar contratos antes de enviar fundos
- 📚 Aprender sobre estrutura Taproot
- 🐛 Debug de problemas de compilação

---

### 2. `verify-puzzle` - Verificar Status do Puzzle

**Uso:**
```bash
cargo run --bin verify-puzzle -- <address_ou_txid>
```

**Exemplos:**
```bash
# Verificar por endereço
cargo run --bin verify-puzzle -- tex1p6k8njks70y4xkv...

# Verificar por TXID da transação de criação
cargo run --bin verify-puzzle -- e7f815d4013f10b8294369c3fff126aef497...

# Verificar por TXID da transação de solução
cargo run --bin verify-puzzle -- 9e47990402fc943ca68f867071da39fd091d...
```

**O que faz:**

#### Verificando por Endereço:
1. ✅ Escaneia blockchain buscando UTXOs no endereço
2. ✅ Mostra se o puzzle está ativo ou foi resolvido
3. ✅ Exibe detalhes do UTXO (txid, vout, valor)
4. ✅ Mostra número de confirmações

**Output - Puzzle Ativo:**
```
🎯 PUZZLE VERIFICATION
=======================

📍 Checking puzzle at address: tex1p6k8njks70y4xkv...

🔎 Scanning blockchain for address: tex1p6k8njks70y4xkv...
   Starting blockchain scan (this may take a moment)...
   Found 1 UTXO(s)

✅ PUZZLE IS ACTIVE
====================

📦 UTXO Details:
   TXID:   e7f815d4013f10b8294369c3fff126aef497...
   VOUT:   1
   Amount: 0.0001 L-BTC
   Asset:  144c654344aa716d...

📋 Transaction Details:
   Confirmations: 5
   Block Time: 1699999999

💡 Status: WAITING TO BE SOLVED
   The puzzle is active and waiting for someone to solve it!
```

**Output - Puzzle Resolvido:**
```
❌ PUZZLE WAS SOLVED OR NEVER FUNDED
=====================================

No UTXO found at this address.

Possible reasons:
  • The puzzle was already solved
  • The puzzle was never funded
  • The transaction is unconfirmed
```

#### Verificando por TXID:
1. ✅ Busca a transação na blockchain
2. ✅ Lista todos os outputs
3. ✅ Verifica quais outputs foram gastos
4. ✅ Identifica transações de solução Simplicity

**Output - Transação de Solução:**
```
🎯 PUZZLE VERIFICATION
=======================

🔍 Checking transaction: 9e47990402fc943ca68f867071da39fd091d...

✅ TRANSACTION FOUND
=====================

📊 Confirmations: 8
📏 Size: 361 bytes

📤 Outputs:
   Output 0: 0.00007 L-BTC
      Address: tlq1qq08dv7jgm7m4jam92lf6wkxvrljrm...
      ⏳ This output is UNSPENT (puzzle active!)
   Output 1: 0.00003 L-BTC
      (Fee output)

📥 Inputs:
   Input 0: e7f815d4013f10b8294369c3fff126aef497...:1
      🔐 Has witness data (4 items)
      💡 This looks like a Simplicity puzzle solution!
```

**Casos de uso:**
- 🔍 Ver se um puzzle ainda está disponível
- 📊 Verificar quantas confirmações tem
- 🏆 Confirmar que você resolveu o puzzle
- 📈 Monitorar status de múltiplos puzzles

---

## 📊 Workflow Completo de Verificação

### 1. Criar Puzzle
```bash
cargo run --bin create-puzzle -- "secret" 0.1 "Hint"
```
Output: `puzzle_abc123.json` e endereço `tex1p...`

### 2. Verificar Contrato (Opcional mas recomendado)
```bash
# Extrair hash do JSON
HASH=$(jq -r '.hash' puzzle_abc123.json)

# Extrair endereço
ADDRESS=$(jq -r '.address' puzzle_abc123.json)

# Verificar compilação
cargo run --bin check-contract -- $HASH $ADDRESS
```
✅ Confirma que o contrato foi compilado corretamente

### 3. Verificar Status Inicial
```bash
cargo run --bin verify-puzzle -- $ADDRESS
```
✅ Confirma que o puzzle foi financiado

### 4. Aguardar Alguém Resolver...

### 5. Verificar se Foi Resolvido
```bash
cargo run --bin verify-puzzle -- $ADDRESS
```
❌ Se mostrar "PUZZLE WAS SOLVED", alguém ganhou!

### 6. Ver Transação de Solução
Se você tem o TXID de uma tentativa de solução:
```bash
cargo run --bin verify-puzzle -- <txid>
```
✅ Mostra detalhes da transação e se teve witness Simplicity

---

## 🔐 O que é CMR (Commitment Merkle Root)?

**CMR** é o hash que representa unicamente um programa Simplicity compilado.

### Como funciona:

1. **Compilação:** Contrato Simplicity → Bytecode
2. **Commitment:** Bytecode → Merkle Tree → CMR (32 bytes)
3. **Endereço:** CMR + Internal Key → Taproot Address

### Por que é importante:

- ✅ **Identifica unicamente** o contrato
- ✅ **Permite verificação** sem revelar o código completo
- ✅ **Fundamental para Taproot** no Bitcoin/Elements
- ✅ **Garante integridade** - se CMR muda, contrato mudou

### Visualização:

```
Secret: "demo"
    ↓ (padding + SHA256)
Hash: 0x20f0e912902bfdc1ea47cdb5eadc6f5c1b3453f406f38dd34a92d0b30a270e22
    ↓ (compila contrato com hash como parâmetro)
Contract Bytecode
    ↓ (commitment)
CMR: 0xb47a817688a2e95f0bf03118c96f9325b1d9bf13ae2739a66ed8a59aa1016d66
    ↓ (Taproot com internal key)
Address: tex1p6k8njks70y4xkv25n2w8vxvapjt8qhx7uen0ncrzr974m6g7xyns78dq6d
```

---

## 🎯 Resumo de Comandos

```bash
# Listar puzzles
cargo run --bin list-puzzles

# Criar puzzle
cargo run --bin create-puzzle -- "secret" 0.1 "Hint"

# Verificar compilação do contrato
cargo run --bin check-contract -- <hash> [address]

# Verificar status do puzzle
cargo run --bin verify-puzzle -- <address_ou_txid>

# Resolver puzzle
cargo run --bin solve-puzzle -- puzzle.json "secret" <destination>

# Adicionar fundos
cargo run --bin add-to-pot -- puzzle.json 0.05
```

---

## 🐛 Troubleshooting

### Problema: "Assertion failed inside jet"
**Causa:** Secret incorreto ou formato errado
**Solução:**
1. Use `solve-puzzle` que verifica antes de gastar
2. Verifique o hash com `check-contract`

### Problema: "UTXO not found"
**Causa:** Puzzle foi resolvido ou transação não confirmada
**Solução:**
1. Use `verify-puzzle` para ver status
2. Aguarde confirmações se recém-criado

### Problema: "Address mismatch" no check-contract
**Causa:** Hash diferente do usado na criação
**Solução:**
1. Verifique se está usando o hash correto do JSON
2. Confirme que o contrato não foi modificado

---

## 📚 Recursos Adicionais

- **Simplicity Language:** https://github.com/BlockstreamResearch/simplicity
- **Jets Specification:** https://github.com/BlockstreamResearch/simplicity/blob/master/jets.md
- **Taproot:** https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki
- **Elements:** https://elementsproject.org/

---

**Happy Puzzle Hunting!** 🎯🔍✨
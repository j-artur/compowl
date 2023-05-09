# Compowl

Implementação de um analisador léxico para um subconjunto da linguagem OWL2 no formato Manchester Syntax.

O programa foi desenvolvido na linguagem de programação [Rust](https://www.rust-lang.org/).

## Compilação

Usando `cargo`:

```console
cargo build
```

O executável estará disponível em `target/debug/compowl`. É recomendado copiá-lo para a pasta raíz para facilitar o acesso.

### OU

usando `rustc`:

```console
rustc src/main.rs -o compowl
```

O executável já estará na pasta raíz com o nome `compowl`.

## Execução

```console
./compowl <output> <file1> <file2> ...
```

#### Argumentos de entrada

1. `<output>`:

- `-f`: para cada arquivo de entrada `<file>` será criado um arquivo de saída `<file>.output`.
- `-t`: cada arquivo de entrada terá a saída exibida no terminal (`stdout`).
- `-ft` | `-tf`: a saída será enviada aos arquivos e ao terminal ao mesmo tempo.

2. `<file1> <file2> ...`:

- Lista de arquivos a serem usados como entrada para o analisador sintático.

## Saída

a saída do programa exibirá todos os tokens identificados pelo analisador léxico se toda a entrada for reconhecida. Caso o analisador encontre algum erro, será exibido no terminal a localização do token não reconhecido, e o arquivo de saída não será criado.

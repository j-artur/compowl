# Compowl

Implementação de um analisador léxico para um subconjunto da linguagem OWL2 no formato Manchester Syntax.

O programa foi desenvolvido na linguagem de programação [Rust](https://www.rust-lang.org/).

## Compilação

Já está incluso no repositório binários executáveis pronto para ser utilizado (`compowl` para linux, `compowl.exe` para windows), mas é possível recompilar de duas formas:

- Usando `cargo`:

  ```console
  cargo build
  ```

  O executável estará disponível em `target/debug/compowl`.

- Usando `rustc`:

  ```console
  rustc src/main.rs -o <name>
  ```

  O executável estará na pasta raíz com o nome indicado.

## Execução

Comando para executar:

```console
./compowl <output> <file1> <file2> <file3> ...
```

### Argumentos de entrada

1. `<output>`:

- `-f`: para cada arquivo de entrada `<file>` será criado um arquivo de saída `<file>.output`.
- `-t`: cada arquivo de entrada terá a saída exibida no terminal (`stdout`).
- `-ft` | `-tf`: a saída será enviada aos arquivos e ao terminal ao mesmo tempo.

2. `<file1> <file2> <file3> ...`:

- Lista de arquivos a serem usados como entrada para o analisador sintático.

### Saída

a saída do programa exibirá todos os tokens identificados pelo analisador léxico se toda a entrada for reconhecida. Caso o analisador encontre algum erro, será exibido no terminal a localização do token não reconhecido, e o arquivo de saída não será criado.

A saída de cada arquivo será no formato:

```
<file>:<line>:<column>: <token1>
<file>:<line>:<column>: <token2>
<file>:<line>:<column>: <token3>
<file>:<line>:<column>: <token4>
...

<file>:<line>:<column>: <decl1_text>
--
<decl1_ast>

<file>:<line>:<column>: <decl2_text>
--
<decl2_ast>

...

<tabela_de_simbolos>

```

### Exemplos

Estão inclusos 6 arquivos de exemplo de entrada e 2 arquivos de saída:

- `file1.txt` -> `file1.txt.output`
- `file2.txt` -> `file2.txt.output`
- `file3.txt`
- `file4.txt`
- `file4.txt`
- `file4.txt`

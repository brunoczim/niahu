# niahu
Niahu: Nova Implementação de Arquiteturas Hipotéticas da UFRGS

# Nota
Este programa foi criado para que alunos do curso de ciência da computação da UFRGS
consigam programar para a máquina hipotética Neander na disciplina de ARQ0, sem estar
no Windows. Ou seja, para alunos que usam Mac e Linux. No entanto, esta implementação
**NÃO** é **OFICIALMENTE** suportada. Portanto, é recomendado que antes de entregar
trabalhos, o arquivo .mem seja testado em uma implementação oficial.

Mais arquiteturas são planejadas para serem implementadas.

# Instalação

## Requisitos:
* git (opcional, para download)
* Toolchain da linguagem Rust, incluindo os programas **rustc** e **cargo**

## Comandos
Não necessitam de super-usuário ou root.

### Download
```shell
git clone https://github.com/brunoczim/niahu/
cd niahu
```

### Neander
Após o download:
```shell
cargo install --path neander
```
Isso instalará um binário chamado `neander` no caminho `~/.cargo/bin/`.
Se você deseja instalá-lo em outro lugar, use:
```shell
cargo install --path neander --root caminho/diretorio/escolhido
```

### Ahmes 
Após o download:
```shell
cargo install --path ahmes
```
Isso instalará um binário chamado `ahmes` no caminho `~/.cargo/bin/`.
Se você deseja instalá-lo em outro lugar, use:
```shell
cargo install --path ahmes --root caminho/diretorio/escolhido
```

### Ramses
Após o download:
```shell
cargo install --path ramses
```
Isso instalará um binário chamado `ahmes` no caminho `~/.cargo/bin/`.
Se você deseja instalá-lo em outro lugar, use:
```shell
cargo install --path ramses --root caminho/diretorio/escolhido
```

# Uso

Além do arquivo .mem habitual, a implementação trabalha com um arquivo .state, que
contém informações mais detalhadas além da memória. Para compartilhar com os colegas
e os professores, você vai querer entregar o arquivo .mem, enquanto o arquivo .state
é usado para fazer execução passo a passo.

## Novo arquivo
Para criar uma memória zerada:
```shell
neander new -o arquivo.mem
```

Para criar um estado zerado:
```shell
neander new -o arquivo.state
```

Em todos os casos, nos exemplos subsequentes onde se passa um arquivo.mem,
arquivo.state também pode ser passado e vice-versa.

Para o Ahmes, todos os comandos são os mesmos, basta substituir `neander` por
`ahmes`.

## Escrever em um Endereço
Para escrever 3 no endereço 50, em decimal:
```shell
neander write -i fonte.mem -o destino.mem -a 50 -d 3
```

Para escrever 3 no endereço A0, em hexadecimal:
```shell
neander write -i fonte.mem -o destino.mem -x -a A0 -d 3
```

Em qualquer subcomando, se fonte for também o destino, somente a fonte
precisa ser especificada. Por exemplo:

```shell
neander write -i fonte_e_destino.mem -a 50 -d 3
```

## Definir o Conteúdo do Program Counter
Para escrever 32 em decimal:
```shell
neander setpc -i fonte.mem -o destino.mem -d 32
```

Para escrever 20 em hexadecimal:
```shell
neander setpc -i fonte.mem -o destino.mem -x -d 20
```

## Executar até o HLT
```shell
neander run -i fonte.mem -o destino.mem
```

## Executar apenas alguns passos:
Para executar 4 passos:
```shell
neander step -i fonte.state -o destino.state -n 4 
```

Se apenas um passo for desejado, `-n` pode ser omitido.

```shell
neander step -i fonte.state -o destino.state
```

Apesar de aceito, um arquivo .mem não terá efeitos na execução.

## Observar a Memória
Em decimal:
```shell
neander data -i fonte.mem -s 128 -e 255
```

Em hexadecimal:
```shell
neander data -i fonte.mem -x -s 80 -e FF
```

Se o intervalo não for especificado, 128--255 será usado.

## Observar a Memória Com Mnemônicos
Em decimal:
```shell
neander code -i fonte.mem -s 0 -e 127
```

Em hexadecimal:
```shell
neander code -i fonte.mem -x -s 0 -e 7F
```

Se o intervalo não for especificado, 0--127 será usado.

## Observar os Registradores
Em decimal:
```shell
neander registers -i fonte.state
```

Em hexadecimal:
```shell
neander registers -i fonte.state -x
```

Note que usar um arquivo .mem devolve registradores zerados, uma vez
que o arquivo contém somente memória.

## Observar Estatísticas
```shell
neander stats -i fonte.state
```
Note que usar um arquivo .mem devolve estatísticas zeradas, uma vez
que o arquivo contém somente memória.

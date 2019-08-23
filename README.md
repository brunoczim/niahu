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

## Escrever em um Endereço
Para escrever 3 no endereço 50, em decimal:
```shell
neander write -i fonte.mem -o destino.mem -a 50 -d 3
```

Para escrever 3 no endereço A0, em hexadecimal:
```shell
neander write -i fonte.mem -o destino.mem -x -a A0 -d 3
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

## Observar a Memória Com Mnemônicos
Em decimal:
```shell
neander code -i fonte.mem -s 0 -e 127
```

Em hexadecimal:
```shell
neander code -i fonte.mem -x -s 0 -e 7F
```

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

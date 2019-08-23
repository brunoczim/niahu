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

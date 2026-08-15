# lispawk

Lisp風の構文で動く、超小型のAWK試作です。現在使えるプログラムは
`(print $1)` だけです。

```console
$ printf 'Alice 20\nBob 30\n' | cargo run --quiet -- '(print $1)'
Alice
Bob
```

空白（スペースやタブ）で各行をフィールドに分割し、第1フィールドを
1行ずつ表示します。

## テスト

```console
$ cargo test
```


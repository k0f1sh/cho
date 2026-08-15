# lispawk

Lisp風の構文で動く、超小型のAWK試作です。

```console
$ printf 'Alice 20\nBob 30\n' | cargo run --quiet -- '(print $2)'
20
30
```

`$0` は行全体をそのまま表示します。`$1` 以上では、空白（スペースやタブ）で
各行をフィールドに分割してN番目を表示します。指定したフィールドが存在しない
行では、空行を出力します。

```console
$ printf 'Alice 20\nBob 30\n' | cargo run --quiet -- '(print $0)'
Alice 20
Bob 30
```

`print` に複数の値を渡すと、スペースで区切って表示します。文字列も値として
利用できます。

```console
$ printf 'Alice 20\nBob 30\n' | cargo run --quiet -- '(print $1 "score:" $2)'
Alice score: 20
Bob score: 30
```

`fmt` は値を区切りなしで結合します。`fmt` 自体は出力せず、結合した文字列を
外側の式へ返します。

```console
$ printf 'Alice 20\nBob 30\n' | cargo run --quiet -- '(print (fmt $1 ":" $2))'
Alice:20
Bob:30
```

トップレベルには式を複数並べられます。各入力行について、式を記述順に実行します。

```console
$ printf 'Alice 20\nBob 30\n' | cargo run --quiet -- $'(print $1)\n(print $2)'
Alice
20
Bob
30
```

## テスト

```console
$ cargo test
```

# lispawk

Lisp風の構文で動く、超小型のAWK試作です。`(print $N)` で各行の
N番目のフィールドを表示できます。

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

## テスト

```console
$ cargo test
```

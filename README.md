# lispawk

Lisp風の構文で動く、超小型のAWK試作です。`(print $N)` で各行の
N番目のフィールドを表示できます。

```console
$ printf 'Alice 20\nBob 30\n' | cargo run --quiet -- '(print $2)'
20
30
```

空白（スペースやタブ）で各行をフィールドに分割します。フィールド番号は
1以上です。指定したフィールドが存在しない行では、空行を出力します。

## テスト

```console
$ cargo test
```

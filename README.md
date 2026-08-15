# cho

AWKに着想を得た、Lisp風のテキスト処理言語です。

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

`NR` は現在の行番号（1始まり）、`NF` は現在行のフィールド数です。通常の値と
同じように `print` や `fmt` の中で使えます。

```console
$ printf 'Alice 20\nBob 30 Osaka\n' | cargo run --quiet -- '(print NR NF $1)'
1 2 Alice
2 3 Bob
```

`filter` の条件が偽なら、現在行に対する残りの式をスキップします。
`>`、`>=`、`<`、`<=` は数値比較で、数値に変換できない値は一致しません。
`=` と `!=` は両辺が数値なら数値として、それ以外なら文字列として比較します。

```console
$ printf 'Alice 18\nBob 30\nCarol 25\n' | \
    cargo run --quiet -- $'(filter (> $2 20))\n(print $1 $2)'
Bob 30
Carol 25
```

式は上から実行されるため、`filter` より前に置いた式はすべての行で実行されます。

```console
$ printf 'Alice 20\nBob 30\n' | \
    cargo run --quiet -- $'(filter (= $1 "Alice"))\n(print $0)'
Alice 20
```

`reg` は正規表現への一致を判定します。引数がパターンだけなら `$0`（行全体）を
対象にし、2引数なら最初の値を対象にします。

```console
$ printf 'info: ready\nerror: failed\n' | \
    cargo run --quiet -- $'(filter (reg "^error:"))\n(print NR $0)'
2 error: failed
```

```lisp
(filter (reg $1 "^[A-Z]"))
(print $1)
```

正規表現は処理開始時に一度だけコンパイルされ、不正なパターンは入力を処理する前に
エラーになります。

## テスト

```console
$ cargo test
```

<details>
<summary>The name</summary>

cho is a Lisp-flavored text processing language inspired by awk.
awk sounds like 「億」 in Japanese, so cho comes next: 「兆」.

</details>

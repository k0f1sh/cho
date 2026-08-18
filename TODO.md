# TODO

choは、小さく組み合わせやすい「Lisp風のsemantic awk」を目指す。

```text
cho is a tiny semantic awk with Lisp-like expressions.
```

文字列、数値、日時、期間、IPアドレス、CIDRなどを、各式のシグネチャが要求する型へ
文脈的に変換する。型コンストラクタを入力のたびに書かせず、変換できない入力は
レコード番号、式名、引数位置、期待型を含む実行時エラーにする。

## 実装済み

- StringとNumberを保持する型付き評価器
- `s/join`、`s/count`、`s/escape`、`s/lower`、`s/upper`による文字列操作
- `print`と`filter`の短縮形`p`と`f`
- 接頭辞なしのstrictな数値比較
- `s/`による文字列比較
- `default`による、局所的な実行時エラーの回復
- DateTimeと`dt/unix`、`dt/fmt`、日時比較
- Durationと`du/s`、`du/m`、`du/h`
- `dt/now`、`dt/add`、`dt/sub`、`dt/diff`
- `dt/floor-s`、`dt/floor-m`、`dt/floor-h`、`dt/floor-d`によるUTC境界への切り下げ
- `ip/private?`、`ip/=`、`ip/!=`、`cidr/contains?`
- 値式用の`->`と`->>`
- 目的別の`examples/`
- LLMエージェント向けの`skills/cho-process-text/`
- `cho --help`の型一覧、式シグネチャ、コピー可能な例

現在の主な規則は次のとおり。

- 入力フィールドはStringであり、式が要求した場合だけNumber、DateTime、Duration、
  IpAddr、Cidrへ変換する。
- `=`、`!=`、`<`、`<=`、`>`、`>=`はNumberを要求する。
- 文字列比較は`s/`、日時比較は`dt/`、IP等値比較は`ip/`を使う。
- DateTime入力はタイムゾーン付きRFC 3339に限定し、既定表示はUTCへ正規化する。
- Durationの既定表示は単位なしの秒数とする。
- `dt/diff LEFT RIGHT`は`LEFT - RIGHT`を返す。
- `ip/private?`はRFC 1918のIPv4だけを真とする。
- 異なるアドレスファミリーの`cidr/contains?`はfalseとする。
- 実行時エラーより前の標準出力は残し、最後は診断と非0終了コードで終える。

## 次に検討するもの

### 型付き値へ渡す直前の文字列抽出

実際のsystemd journalをchoで処理したところ、`short-iso-precise`の先頭フィールドは
タイムゾーン付きRFC 3339なので、日時処理は前処理なしでそのまま使えた。

```sh
journalctl --no-pager -n 200 -o short-iso-precise |
  cho '
    (f (dt/>= $1 (dt/sub (dt/now) (du/m 10))))
    (f (~ $3 /^tailscaled/))
    (p (dt/fmt "%H:%M:%S" $1) $3)
  '
```

一方、同じログ内のIPアドレスは`192.168.10.20:39652`、`SRC=10.0.0.25`、
`[fd00::1]:443`のように、ポート、接頭辞、区切り文字を伴うことが多い。
`ip/private?`へ`IP:port`をそのまま渡すと、strictな変換規則により意図どおり停止した。

```text
cho: record 155: ip/private?: argument 1 expects IpAddr,
but "192.168.10.20:39652" is not a valid IPv4 or IPv6 address
```

現在は外部コマンドで日時、送信元IP、宛先IPをTSVへ取り出せば、その後の意味的な処理を
choへ任せられる。

```sh
journalctl -u tailscaled --no-pager -n 200 -o short-iso-precise |
  sed -nE 's/^([^ ]+) .*open-conn-track: timeout opening \(TCP ([0-9.]+):[0-9]+ => ([0-9.]+):[0-9]+\).*/\1\t\2\t\3/p' |
  cho --tsv '
    (f (dt/>= $1 (dt/sub (dt/now) (du/m 10))))
    (f (cidr/contains? "192.168.0.0/16" $2))
    (f (ip/private? $3))
    (p (dt/fmt "%H:%M:%S" $1) (s/join " -> " $2 $3))
  '
```

この前処理をcho内で小さく書けるよう、文字列から一部分を取り出す直交した値式を検討
する。候補は、区切り文字と位置を指定する分割式、正規表現の一致部分またはキャプチャを
返す式など。単に`s/split`を追加すると配列型やIPv6のコロンをどう扱うかまで設計が広がる
ため、最初は「1つのStringを返す」最小の抽出操作を優先する。

決める必要がある事項：

- 区切り文字による抽出と正規表現キャプチャのどちらを最初に提供するか
- 一致しない場合を空文字にするか、strictな実行時エラーにするか
- キャプチャ番号が存在しない場合の診断
- 正規表現リテラルと動的なパターンの扱い
- IPv4の`IP:port`、IPv6の`[IP]:port`、`SRC=IP`を小さな式で扱えるか
- `default`、`str`、`s/join`、`ip/`、`cidr/`の内外へ自然にネストできるか

実装する場合は、外部の`sed`を使った上記コマンドと、choだけで完結するコマンドを
examplesへ並べ、strictな失敗と`default`による明示的な回復も確認する。

### Durationの任意表示

Durationはそのまま出力すれば秒数になる。人向けの表示が本当に必要な実例を集めてから、
`du/fmt`の書式を決める。

```lisp
(du/fmt FORMAT (dt/diff $1 $2))
```

決める必要がある事項：

- 時分秒の各要素と総時間をどう区別するか
- 24時間を超える期間と負の期間の表示
- 小数秒の精度と末尾ゼロ
- FORMATを自由な書式文字列にするか、少数の名前付き形式にするか

### IP分類述語

実際の用途を確認し、必要なものだけを追加する。

- `ip/loopback?`
- `ip/link-local?`
- `ip/multicast?`
- `ip/v4?`
- `ip/v6?`
- IPv6 unique-local用の述語

順序比較は当面追加しない。

### その他の型付き値

Date、URL、URI、JSONを候補とする。各型について、正規化、既定表示、比較、失敗時の
診断、最小の型固有操作を決めてから一つずつ実装する。

```lisp
(date/> $1 "2026-08-01")
(url/host $1)
```

文字列の見た目から型を自動判定せず、`date/`、`url/`、`uri/`、`json/`の式が期待型を
決める現在の規則を維持する。

## 当面扱わないもの

- 曖昧な日付形式やローカルタイムゾーンの暗黙利用
- Unix秒・ミリ秒・マイクロ秒・ナノ秒の桁数による自動判定
- cho内部でのソート
- 一般的なマクロ定義
- `BEGIN`、`END`、レコードをまたぐ変更可能な状態

## 変更時の確認

```console
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

ユーザー向け構文を変更した場合は、実バイナリの出力、標準エラー、終了コードを確認し、
`cho --help`、README、対応するexample、skillのワークフローが矛盾しないことも確認する。

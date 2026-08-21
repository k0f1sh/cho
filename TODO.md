# TODO

choは、小さく組み合わせやすい「Lisp風のsemantic awk」を目指す。

```text
cho is a tiny semantic awk with Lisp-like expressions.
```

文字列、数値、日時、期間、IPアドレス、CIDRなどを、各式のシグネチャが要求する型へ
文脈的に変換する。型コンストラクタを入力のたびに書かせず、変換できない入力は
レコード番号、式名、引数位置、期待型を含む実行時エラーにする。

###  現在の主な規則は次のとおり。

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

### 型付きデータ操作の強化

以下は、一項目ずつ実装、テスト、文書更新、実バイナリでの確認まで完了させてから
次へ進む。新しい式は通常の値式として実装し、`print`、`if`、`default`や他の型付き式へ
ネストできる状態を完了条件とする。

#### 1. 型付きextractorの土台と安全な整数変換

- `cidr/network`が返すIpAddrを文字列へ戻さず、後続のIP式へ渡せる内部表現を整える。
- 現在の`Number`は`f64`なので、整数を返す式で精度損失を黙って起こさない共通規則を
  決める。
- `u64`などからNumberへ変換する場合は、JavaScriptと同じ安全整数上限
  `2^53 - 1`を基準とし、範囲外は回復可能なruntime errorにする案を第一候補とする。
- エラーには従来どおりrecord、expression、argument number、期待型を含める。

#### 2. Durationの単位追加（実装済み）

```lisp
(du/ms NUMBER)
(du/d NUMBER)
```

- `du/ms`はmilliseconds、`du/d`は厳密に24時間単位とする。
- `du/s`、`du/m`、`du/h`と同じ負数、小数、rounding、overflow規則を使う。
- monthやyearのような可変長単位は追加しない。
- AST、parser、runtime、help、README、引数検証とruntime testを更新済み。

#### 3. IP versionとprivateのIPv6対応

```lisp
(ip/version IPADDR)
```

- IPv4なら`4`、IPv6なら`6`をNumberとして返す。
- `ip/private?`はRFC 1918のIPv4に加え、IPv6 ULA `fc00::/7`をtrueとする。
- loopback、link-local、multicastは既存の専用predicateに委ね、privateには含めない。
- helpとREADMEにprivateの範囲を明記する。

#### 4. CIDRの基本情報抽出

```lisp
(cidr/network CIDR)
(cidr/prefix CIDR)
(cidr/first CIDR)
(cidr/last CIDR)
```

- `network`と`first`はいずれもCIDR範囲の最小アドレスを返す。API上は役割を明示する
  extractorとして両方を提供する。
- `last`はusable hostではなく純粋な最大アドレスとし、IPv4のbroadcast addressも返す。
- IPv4/IPv6、host bitsが立った入力、`/0`、`/32`、`/128`、不正入力をテストする。
- 戻り値のIpAddrを`ip/version`やIP predicateへ直接ネストできることを確認する。

#### 5. CIDRのアドレス数

```lisp
(cidr/size CIDR)
```

- 総アドレス数をNumberで正確に表現できる場合だけ返す。
- 安全整数上限を超える場合はruntime errorとし、`default`で回復可能にする。
- IPv4/IPv6の通常ケース、最大成功境界、その直前のoverflow、IPv6 `/0`をテストする。

#### 6. URL query parameterの取得

```lisp
(url/query-get NAME URL)
(url/query-has? NAME URL)
```

- keyとvalueは`application/x-www-form-urlencoded`のquery semanticsでdecodeし、`+`は
  spaceとして扱う。
- keyはdecode後のNAMEと比較する。
- 同名keyが複数ある場合、`query-get`は最初の値を返す。
- 存在しないkeyの値は空文字とする。`?foo`と`?foo=`はいずれも存在すると判定し、
  値は空文字とする。
- normal、missing、empty、percent encoding、`+`、duplicate、queryなし、不正URLを
  テストする。
- 既存の`url/query`はraw query stringを返す式として変更しない。

#### 7. SemVer componentの抽出

```lisp
(semver/major SEMVER)
(semver/minor SEMVER)
(semver/patch SEMVER)
(semver/prerelease SEMVER)
```

- prereleaseがない場合は空文字を返す。
- build metadataを含む入力も既存parserどおり受け付け、今回build extractorは追加しない。
- major、minor、patchがNumberの安全整数範囲を超える場合はruntime errorとし、丸めない。
- 通常、prereleaseあり・なし、build metadata、不正SemVer、安全整数境界をテストする。

#### 8. Boolean expressionとしてのpredicateの文書整理

- predicateはBooleanを返す通常のvalue expressionであり、`filter`と`if`がBooleanを
  要求する、という利用者向けモデルをhelpで明確にする。
- 現在の`Predicate` ASTと`Value::Predicate`で合成できているため、大規模な内部
  リファクタリングは行わない。
- predicateを`print`、`if`、`filter`で使うテストを維持・補強する。
- helpは型ごとにconstructor/conversion、extraction、predicate/operationを探しやすく
  整理するが、既存の式名や分類を不必要に動かさない。

各項目の完了時には、AST表現、再帰的な値構文、引数不足・過剰のparser test、runtime
test、`default`によるexpected runtime errorの回復、`cho --help`とREADMEの小さな例を
確認する。関連する既存exampleや`skills/cho-process-text/`の説明に影響がある場合だけ、
同時に更新する。

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

- `ip/v4?`
- `ip/v6?`

IPv6 unique-localは、まず上記の`ip/private?`で扱う。独立した`ip/unique-local?`は、
privateと区別する実例が集まるまで追加しない。

順序比較は当面追加しない。

### その他の型付き値

ログや運用データによく現れ、文字列のままでは正しく扱いにくい値を候補とする。
汎用的なデータ加工機能を増やすのではなく、値の意味を保ったまま短い式で扱えることを
choの強みにする。

優先して検討する候補：

- ByteSize：`10MiB`と`500KB`の比較、単位変換
- UUID：検証、正規化、等値比較
- MACアドレス：表記揺れを吸収した等値比較
- Date：時刻を伴わない日付の比較
- JSON：ログフィールド内の値の抽出。ただし言語規模への影響を慎重に見る

```lisp
(bs/> $1 "100MiB")
(sv/>= $2 "2.4.0")
(date/> $1 "2026-08-01")
(url/host $1)
```

各型は、次を満たすか検討してから一つずつ実装する。

- 文字列比較では意味的に誤る、またはシェルだけで正しく扱うのが難しい
- 検証だけでなく、比較、抽出、変換など複数の有用な操作を定義できる
- 文脈変換によって記述が明確に短くなる
- 外部I/Oや変更可能なグローバル状態を必要とせず、値式として合成できる
- 型名、式名、既定表示、失敗時の診断を`cho --help`で把握できる

各型について、正規化、既定表示、比較、失敗時の診断、最小の型固有操作を決める。
文字列の見た目から型を自動判定せず、`bs/`、`sv/`、`date/`、`url/`など各名前空間の
式が期待型を決める現在の規則を維持する。

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

use super::support::output;
use std::io::Cursor;

#[test]
fn ip_and_cidr_predicates_are_typed() {
    assert_eq!(
        output(
            "(filter (ip/private? $1)) (print $1)",
            "10.1.2.3\n8.8.8.8\nfc00::1\n"
        ),
        "10.1.2.3\nfc00::1\n"
    );
    assert_eq!(
        output(
            r#"(filter (cidr/contains? "10.0.0.0/8" $1)) (print $1)"#,
            "10.2.3.4\n11.0.0.1\n2001:db8::1\n"
        ),
        "10.2.3.4\n"
    );
    assert_eq!(
        output(
            r#"(filter (ip/= $1 "2001:db8::1")) (print $1)"#,
            "2001:0db8:0:0:0:0:0:1\n2001:db8::2\n"
        ),
        "2001:0db8:0:0:0:0:0:1\n"
    );
}

#[test]
fn ip_version_predicates_distinguish_ipv4_and_ipv6() {
    assert_eq!(
        output(
            "(filter (ip/v4? $1)) (print $1)",
            "192.0.2.1\n::ffff:192.0.2.1\n2001:db8::1\n"
        ),
        "192.0.2.1\n"
    );
    assert_eq!(
        output(
            "(filter (ip/v6? $1)) (print $1)",
            "192.0.2.1\n::ffff:192.0.2.1\n2001:db8::1\n"
        ),
        "::ffff:192.0.2.1\n2001:db8::1\n"
    );
    assert_eq!(
        output(
            "(print (if (ip/v4? (cidr/network $1)) \"v4\" \"v6\"))",
            "10.0.0.0/8\n2001:db8::/32\n"
        ),
        "v4\nv6\n"
    );
}

#[test]
fn ip_version_predicates_report_their_own_conversion_errors() {
    for predicate in ["ip/v4?", "ip/v6?"] {
        let program = format!("(filter ({predicate} $1))");
        let error = cho::run(&program, Cursor::new("not-an-ip\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with(&format!("record 1: {predicate}: argument 1 expects IpAddr")),
            "{error}"
        );
        assert_eq!(
            output(
                &format!("(print (default ({predicate} $2) \"invalid\"))"),
                "192.0.2.1\n"
            ),
            "invalid\n"
        );
    }
}

#[test]
fn private_ip_predicate_includes_ipv6_unique_local_boundaries() {
    assert_eq!(
        output(
            "(filter (ip/private? $1)) (print $1)",
            concat!(
                "10.0.0.0\n172.16.0.0\n172.31.255.255\n192.168.255.255\n",
                "9.255.255.255\n172.32.0.0\n192.169.0.0\n",
                "fc00::\nfdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff\nfbff::1\nfe00::1\n",
            ),
        ),
        concat!(
            "10.0.0.0\n172.16.0.0\n172.31.255.255\n192.168.255.255\n",
            "fc00::\nfdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff\n",
        )
    );
}

#[test]
fn cidr_extractors_return_ipv4_and_ipv6_boundaries() {
    assert_eq!(
        output(
            concat!(
                "(print (cidr/network $1) (cidr/prefix $1) ",
                "(cidr/first $1) (cidr/last $1))",
            ),
            concat!(
                "192.168.1.42/24\n0.0.0.0/0\n192.0.2.1/32\n",
                "2001:db8::42/64\n::/0\n2001:db8::1/128\n",
            ),
        ),
        concat!(
            "192.168.1.0 24 192.168.1.0 192.168.1.255\n",
            "0.0.0.0 0 0.0.0.0 255.255.255.255\n",
            "192.0.2.1 32 192.0.2.1 192.0.2.1\n",
            "2001:db8:: 64 2001:db8:: 2001:db8::ffff:ffff:ffff:ffff\n",
            ":: 0 :: ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff\n",
            "2001:db8::1 128 2001:db8::1 2001:db8::1\n",
        )
    );
}

#[test]
fn cidr_ip_results_remain_typed_and_compose_with_ip_functions() {
    assert_eq!(
        output(
            concat!(
                "(print (ip/version (cidr/network $1)) ",
                "(ip/private? (cidr/first $1)) ",
                "(ip/= (cidr/last $1) $2))",
            ),
            "10.1.2.3/24 10.1.2.255\n2001:db8::1/126 2001:db8::3\n",
        ),
        "4 true true\n6 false true\n"
    );
}

#[test]
fn cidr_extractors_report_and_recover_from_conversion_errors() {
    for extractor in ["cidr/network", "cidr/prefix", "cidr/first", "cidr/last"] {
        let program = format!("(print ({extractor} $1))");
        let error = cho::run(&program, Cursor::new("not-a-cidr\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with(&format!("record 1: {extractor}: argument 1 expects Cidr")),
            "{error}"
        );
    }
    assert_eq!(
        output(
            "(print (default (cidr/network $1) \"invalid\"))",
            "not-a-cidr\n"
        ),
        "invalid\n"
    );
}

#[test]
fn cidr_size_is_exact_or_reports_a_recoverable_overflow() {
    assert_eq!(
        output(
            "(print (cidr/size $1))",
            "0.0.0.0/0\n192.0.2.1/32\n2001:db8::/76\n2001:db8::1/128\n"
        ),
        "4294967296\n1\n4503599627370496\n1\n"
    );
    let error = cho::run(
        "(print (cidr/size $1))",
        Cursor::new("2001:db8::/75\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("record 1: cidr/size: argument 1 expects"),
        "{error}"
    );
    assert_eq!(
        output("(print (default (cidr/size $1) \"too-large\"))", "::/0\n"),
        "too-large\n"
    );
}

#[test]
fn ip_classification_predicates_cover_ipv4_and_ipv6_boundaries() {
    assert_eq!(
        output(
            "(filter (ip/loopback? $1)) (print $1)",
            "127.0.0.1\n127.255.255.255\n126.255.255.255\n::1\n::2\n",
        ),
        "127.0.0.1\n127.255.255.255\n::1\n"
    );
    assert_eq!(
        output(
            "(filter (ip/link-local? $1)) (print $1)",
            concat!(
                "169.254.0.1\n169.254.255.255\n169.253.255.255\n",
                "fe80::1\nfebf::1\nfec0::1\n",
            ),
        ),
        "169.254.0.1\n169.254.255.255\nfe80::1\nfebf::1\n"
    );
    assert_eq!(
        output(
            "(filter (ip/multicast? $1)) (print $1)",
            concat!(
                "224.0.0.1\n239.255.255.255\n223.255.255.255\n240.0.0.1\n",
                "ff02::1\nfeff::1\n",
            ),
        ),
        "224.0.0.1\n239.255.255.255\nff02::1\n"
    );
}

#[test]
fn ip_classification_predicates_report_their_own_conversion_errors() {
    for predicate in ["ip/loopback?", "ip/link-local?", "ip/multicast?"] {
        let program = format!("(filter ({predicate} $1))");
        let error = cho::run(&program, Cursor::new("not-an-ip\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with(&format!("record 1: {predicate}: argument 1 expects IpAddr")),
            "{error}"
        );
    }
}

#[test]
fn default_can_recover_from_an_invalid_ip() {
    assert_eq!(
        output(
            r#"(print (default (if (ip/private? $1) "private" "public") "invalid"))"#,
            "10.0.0.1\nnot-an-ip\n2001:db8::1\n"
        ),
        "private\ninvalid\npublic\n"
    );
}

#[test]
fn ip_version_returns_a_number_and_composes_as_a_value() {
    assert_eq!(
        output(
            "(print $1 (ip/version $1) (if (= (ip/version $1) 4) \"v4\" \"v6\"))",
            "192.0.2.1\n2001:db8::1\n"
        ),
        "192.0.2.1 4 v4\n2001:db8::1 6 v6\n"
    );
    assert_eq!(
        output(
            "(print (default (ip/version $1) \"invalid\"))",
            "not-an-ip\n"
        ),
        "invalid\n"
    );
}

#[test]
fn ip_normalization_and_formats_cover_both_families() {
    assert_eq!(
        output(
            "(p (ip $1) (ip/expand (ip $1)))",
            "192.0.2.1\n2001:0DB8:0:0:0:0:0:1\n::ffff:192.0.2.1\n"
        ),
        concat!(
            "192.0.2.1 192.0.2.1\n",
            "2001:db8::1 2001:0db8:0000:0000:0000:0000:0000:0001\n",
            "::ffff:192.0.2.1 0000:0000:0000:0000:0000:ffff:c000:0201\n"
        )
    );
    assert_eq!(
        output("(ip/binary (ip $1))", "192.0.2.1\n"),
        "11000000.00000000.00000010.00000001\n"
    );
    assert_eq!(
        output("(ip/binary $1)", "2001:db8::1\n"),
        concat!(
            "0010000000000001:0000110110111000:",
            "0000000000000000:0000000000000000:0000000000000000:",
            "0000000000000000:0000000000000000:0000000000000001\n"
        )
    );
    assert_eq!(
        output("(ip/binary $1)", "255.255.255.255\n"),
        format!("{}\n", ["11111111"; 4].join("."))
    );
    assert_eq!(
        output("(ip/binary $1)", "::\n"),
        format!("{}\n", ["0000000000000000"; 8].join(":"))
    );
    assert_eq!(
        output(
            "(ip/binary $1)",
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff\n"
        ),
        format!("{}\n", ["1111111111111111"; 8].join(":"))
    );
}

#[test]
fn cidr_constructors_normalize_and_remain_typed() {
    assert_eq!(
        output(
            "(p (cidr $1) (cidr (cidr/network $1) (cidr/prefix $1)) (cidr/from-mask (cidr/last $1) (cidr/netmask $1)))",
            "192.168.1.42/24\n2001:db8::42/64\n"
        ),
        "192.168.1.0/24 192.168.1.0/24 192.168.1.0/24\n2001:db8::/64 2001:db8::/64 2001:db8::/64\n"
    );
    assert_eq!(
        output(
            "(p (cidr/contains? (cidr $1) (ip $2)) (ip/version (cidr/host-first (cidr $1))) (cidr/prefix (cidr $1)) (cidr/first (cidr $1)) (cidr/last (cidr $1)) (cidr/size (cidr $1)))",
            "192.168.1.42/24 192.168.1.255\n2001:db8::42/126 2001:db8::44\n"
        ),
        "true 4 24 192.168.1.0 192.168.1.255 256\nfalse 6 126 2001:db8::40 2001:db8::43 4\n"
    );
    assert_eq!(
        output(
            "(p (cidr (ip $1) (+ 20 4)) (str (default (cidr $1 24) \"bad\")) (s/join \",\" (cidr $1 24) (cidr $1 32)))",
            "192.168.1.42\n"
        ),
        "192.168.1.0/24 192.168.1.0/24 192.168.1.0/24,192.168.1.42/32\n"
    );
    assert_eq!(
        output("(-> $1 (ip) (cidr 24) (cidr/host-count))", "192.168.1.42\n"),
        "254\n"
    );
}

#[test]
fn network_masks_round_trip_every_prefix() {
    for (ip, bits) in [("192.168.1.42", 32), ("2001:db8::42", 128)] {
        for prefix in 0..=bits {
            let input = format!("{ip}/{prefix}\n");
            assert_eq!(
                output("(cidr/from-mask (cidr/last $1) (cidr/netmask $1))", &input),
                output("(cidr $1)", &input),
                "{input}"
            );
            assert_eq!(
                output(&format!("(cidr \"{ip}\" {prefix})"), "\n"),
                output("(cidr $1)", &input),
                "{input}"
            );
        }
    }
    assert_eq!(
        output(
            "(p (cidr/netmask $1) (cidr/wildcard $1))",
            "192.168.1.42/24\n0.0.0.0/0\n192.0.2.1/32\n2001:db8::42/64\n::/0\n::1/128\n"
        ),
        concat!(
            "255.255.255.0 0.0.0.255\n0.0.0.0 255.255.255.255\n255.255.255.255 0.0.0.0\n",
            "ffff:ffff:ffff:ffff:: ::ffff:ffff:ffff:ffff\n:: ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff\n",
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff ::\n"
        )
    );
}

#[test]
fn host_ranges_and_exact_counts_cover_boundaries() {
    assert_eq!(
        output(
            "(p (cidr/host-first $1) (cidr/host-last $1) (cidr/host-count $1) (cidr/size-str $1))",
            "192.168.1.42/24\n0.0.0.0/0\n192.0.2.1/30\n192.0.2.1/31\n192.0.2.1/32\n2001:db8::42/64\n2001:db8::1/127\n2001:db8::1/128\n::/0\n"
        ),
        concat!(
            "192.168.1.1 192.168.1.254 254 256\n",
            "0.0.0.1 255.255.255.254 4294967294 4294967296\n",
            "192.0.2.1 192.0.2.2 2 4\n",
            "192.0.2.0 192.0.2.1 2 2\n",
            "192.0.2.1 192.0.2.1 1 1\n",
            "2001:db8:: 2001:db8::ffff:ffff:ffff:ffff 18446744073709551616 18446744073709551616\n",
            "2001:db8:: 2001:db8::1 2 2\n",
            "2001:db8::1 2001:db8::1 1 1\n",
            ":: ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff 340282366920938463463374607431768211456 340282366920938463463374607431768211456\n"
        )
    );
    assert_eq!(
        output(
            "(p (cidr/size-str $1) (cidr/host-count $1))",
            "::/1\n::/75\n"
        ),
        "170141183460469231731687303715884105728 170141183460469231731687303715884105728\n9007199254740992 9007199254740992\n"
    );
    // These functions return String even when the count would fit Number.
    assert_eq!(
        output(
            "(p (s/count (cidr/host-count $1)) (s/join \":\" (cidr/size-str $1) \"addresses\"))",
            "192.168.1.42/24\n"
        ),
        "3 256:addresses\n"
    );
}

#[test]
fn new_network_functions_reject_missing_empty_and_malformed_values() {
    for (name, expected_type) in [
        ("ip", "IpAddr"),
        ("ip/expand", "IpAddr"),
        ("ip/binary", "IpAddr"),
        ("cidr", "Cidr"),
        ("cidr/netmask", "Cidr"),
        ("cidr/wildcard", "Cidr"),
        ("cidr/host-first", "Cidr"),
        ("cidr/host-last", "Cidr"),
        ("cidr/host-count", "Cidr"),
        ("cidr/size-str", "Cidr"),
    ] {
        for arg in ["$2", "\"\"", "\"invalid\"", "true", "42"] {
            let program = format!("({name} {arg})");
            let error = cho::run(&program, Cursor::new("record\n"), Vec::new()).unwrap_err();
            assert!(
                error.to_string().starts_with(&format!(
                    "record 1: {name}: argument 1 expects {expected_type}"
                )),
                "{program}: {error}"
            );
            assert_eq!(
                output(&format!("(default ({name} {arg}) \"bad\")"), "record\n"),
                "bad\n"
            );
        }
    }
}

#[test]
fn cidr_constructors_report_the_invalid_argument() {
    for (program, function, argument) in [
        ("(cidr \"invalid\" 24)", "cidr", 1),
        ("(cidr $2 24)", "cidr", 1),
        ("(cidr \"\" 24)", "cidr", 1),
        ("(cidr \"192.0.2.1\" $2)", "cidr", 2),
        ("(cidr \"192.0.2.1\" \"\")", "cidr", 2),
        ("(cidr \"192.0.2.1\" 33)", "cidr", 2),
        ("(cidr \"::1\" 129)", "cidr", 2),
        ("(cidr \"::1\" -1)", "cidr", 2),
        ("(cidr \"::1\" 64.5)", "cidr", 2),
        ("(cidr \"::1\" true)", "cidr", 2),
        ("(cidr \"::1\" \"NaN\")", "cidr", 2),
        ("(cidr \"::1\" \"inf\")", "cidr", 2),
        ("(cidr/from-mask $2 \"255.255.255.0\")", "cidr/from-mask", 1),
        (
            "(cidr/from-mask \"\" \"255.255.255.0\")",
            "cidr/from-mask",
            1,
        ),
        ("(cidr/from-mask \"192.0.2.1\" $2)", "cidr/from-mask", 2),
        ("(cidr/from-mask \"192.0.2.1\" \"\")", "cidr/from-mask", 2),
        (
            "(cidr/from-mask \"192.0.2.1\" \"255.0.255.0\")",
            "cidr/from-mask",
            2,
        ),
        (
            "(cidr/from-mask \"192.0.2.1\" \"0.0.0.255\")",
            "cidr/from-mask",
            2,
        ),
        (
            "(cidr/from-mask \"::1\" \"ffff:fffe:ffff::\")",
            "cidr/from-mask",
            2,
        ),
        (
            "(cidr/from-mask \"::1\" \"255.255.255.0\")",
            "cidr/from-mask",
            2,
        ),
        (
            "(cidr/from-mask \"192.0.2.1\" \"ffff::\")",
            "cidr/from-mask",
            2,
        ),
    ] {
        let error = cho::run(program, Cursor::new("record\n"), Vec::new()).unwrap_err();
        assert!(
            error.to_string().starts_with(&format!(
                "record 1: {function}: argument {argument} expects"
            )),
            "{program}: {error}"
        );
        assert_eq!(
            output(&format!("(default {program} \"bad\")"), "record\n"),
            "bad\n"
        );
    }
    // Host-only input and dotted masks are not implicit CIDR syntax.
    for input in ["192.0.2.1\n", "192.0.2.1/255.255.255.0\n"] {
        assert_eq!(output("(default (cidr $1) \"bad\")", input), "bad\n");
    }
}

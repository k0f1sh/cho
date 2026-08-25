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

use adstxt::*;

#[test]
fn test_placeholder() {
    assert_eq!(AdsTxt::parse_lines("placeholder.example.com, placeholder, DIRECT, placeholder"), vec![
        LineData::Record {
            record: Record {
                domain: "placeholder.example.com",
                account_id: "placeholder",
                relation: Relation::Direct,
                authority_id: Some("placeholder"),
            },
            extension: None,
            comment: None
        }
    ]);
}

#[test]
fn test_single_system_direct() {
    assert_eq!(AdsTxt::parse_lines("greenadexchange.com, XF7342, DIRECT, 5jyxf8k54"), vec![LineData::Record {
        record: Record {
            domain: "greenadexchange.com",
            account_id: "XF7342",
            relation: Relation::Direct,
            authority_id: Some("5jyxf8k54"),
        },
        extension: None,
        comment: None
    }]);
}

#[test]
fn test_single_system_reseller() {
    assert_eq!(AdsTxt::parse_lines("redssp.com, 57013, RESELLER"), vec![LineData::Record {
        record: Record { domain: "redssp.com", account_id: "57013", relation: Relation::Reseller, authority_id: None },
        extension: None,
        comment: None
    }]);
}

#[test]
fn test_multiple_systems_and_resellers() {
    assert_eq!(
        AdsTxt::parse_lines(
            r###"# Ads.txt file for example.com:
greenadexchange.com, 12345, DIRECT, d75815a79
silverssp.com, 9675, RESELLER, f496211
blueadexchange.com, XF436, DIRECT
orangeexchange.com, 45678, RESELLER
silverssp.com, ABE679, RESELLER"###
        ),
        vec![
            LineData::Comment(Comment("# Ads.txt file for example.com:")),
            LineData::Record {
                record: Record {
                    domain: "greenadexchange.com",
                    account_id: "12345",
                    relation: Relation::Direct,
                    authority_id: Some("d75815a79"),
                },
                extension: None,
                comment: None
            },
            LineData::Record {
                record: Record {
                    domain: "silverssp.com",
                    account_id: "9675",
                    relation: Relation::Reseller,
                    authority_id: Some("f496211"),
                },
                extension: None,
                comment: None
            },
            LineData::Record {
                record: Record {
                    domain: "blueadexchange.com",
                    account_id: "XF436",
                    relation: Relation::Direct,
                    authority_id: None,
                },
                extension: None,
                comment: None
            },
            LineData::Record {
                record: Record {
                    domain: "orangeexchange.com",
                    account_id: "45678",
                    relation: Relation::Reseller,
                    authority_id: None,
                },
                extension: None,
                comment: None
            },
            LineData::Record {
                record: Record {
                    domain: "silverssp.com",
                    account_id: "ABE679",
                    relation: Relation::Reseller,
                    authority_id: None,
                },
                extension: None,
                comment: None
            },
        ]
    );
}

#[test]
fn test_contact_records() {
    assert_eq!(
        AdsTxt::parse_lines(
            r###"# Ads.txt file for example.com:
greenadexchange.com, 12345, DIRECT, d75815a79
blueadexchange.com, XF436, DIRECT
contact=adops@example.com
contact=http://example.com/contact-us"###
        ),
        vec![
            LineData::Comment(Comment("# Ads.txt file for example.com:")),
            LineData::Record {
                record: Record {
                    domain: "greenadexchange.com",
                    account_id: "12345",
                    relation: Relation::Direct,
                    authority_id: Some("d75815a79"),
                },
                extension: None,
                comment: None
            },
            LineData::Record {
                record: Record {
                    domain: "blueadexchange.com",
                    account_id: "XF436",
                    relation: Relation::Direct,
                    authority_id: None,
                },
                extension: None,
                comment: None
            },
            LineData::Variable {
                variable: Variable { name: "contact", value: "adops@example.com" },
                extension: None,
                comment: None
            },
            LineData::Variable {
                variable: Variable { name: "contact", value: "http://example.com/contact-us" },
                extension: None,
                comment: None
            },
        ]
    );
}

#[test]
fn test_subdomain_referral() {
    assert_eq!(
        AdsTxt::parse_lines(
            r###"# Ads.txt file for example.com:
greenadexchange.com, 12345, DIRECT, d75815a79
blueadexchange.com, XF436, DIRECT
subdomain=divisionone.example.com"###
        ),
        vec![
            LineData::Comment(Comment("# Ads.txt file for example.com:")),
            LineData::Record {
                record: Record {
                    domain: "greenadexchange.com",
                    account_id: "12345",
                    relation: Relation::Direct,
                    authority_id: Some("d75815a79"),
                },
                extension: None,
                comment: None
            },
            LineData::Record {
                record: Record {
                    domain: "blueadexchange.com",
                    account_id: "XF436",
                    relation: Relation::Direct,
                    authority_id: None,
                },
                extension: None,
                comment: None
            },
            LineData::Variable {
                variable: Variable { name: "subdomain", value: "divisionone.example.com" },
                extension: None,
                comment: None
            },
        ]
    );
}

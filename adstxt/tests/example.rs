use adstxt::parser::{parse, Record, Relation, Row, Variable};

#[test]
fn test_placeholder() {
    assert_eq!(
        parse("placeholder.example.com, placeholder, DIRECT, placeholder").unwrap(),
        vec![Row::Record(Record {
            domain: "placeholder.example.com",
            account_id: "placeholder",
            relation: Relation::Direct,
            authority_id: Some("placeholder"),
        })]
    );
}

#[test]
fn test_single_system_direct() {
    assert_eq!(
        parse("greenadexchange.com, XF7342, DIRECT, 5jyxf8k54").unwrap(),
        vec![Row::Record(Record {
            domain: "greenadexchange.com",
            account_id: "XF7342",
            relation: Relation::Direct,
            authority_id: Some("5jyxf8k54"),
        })]
    );
}

#[test]
fn test_single_system_reseller() {
    assert_eq!(
        parse("redssp.com, 57013, RESELLER").unwrap(),
        vec![Row::Record(Record {
            domain: "redssp.com",
            account_id: "57013",
            relation: Relation::Reseller,
            authority_id: None,
        })]
    );
}

#[test]
fn test_multiple_systems_and_resellers() {
    assert_eq!(
        parse(
            r###"# Ads.txt file for example.com:
greenadexchange.com, 12345, DIRECT, d75815a79
silverssp.com, 9675, RESELLER, f496211
blueadexchange.com, XF436, DIRECT
orangeexchange.com, 45678, RESELLER
silverssp.com, ABE679, RESELLER"###
        )
        .unwrap(),
        vec![
            Row::Comment("Ads.txt file for example.com:"),
            Row::Record(Record {
                domain: "greenadexchange.com",
                account_id: "12345",
                relation: Relation::Direct,
                authority_id: Some("d75815a79"),
            }),
            Row::Record(Record {
                domain: "silverssp.com",
                account_id: "9675",
                relation: Relation::Reseller,
                authority_id: Some("f496211"),
            }),
            Row::Record(Record {
                domain: "blueadexchange.com",
                account_id: "XF436",
                relation: Relation::Direct,
                authority_id: None,
            }),
            Row::Record(Record {
                domain: "orangeexchange.com",
                account_id: "45678",
                relation: Relation::Reseller,
                authority_id: None,
            }),
            Row::Record(Record {
                domain: "silverssp.com",
                account_id: "ABE679",
                relation: Relation::Reseller,
                authority_id: None,
            }),
        ]
    );
}

#[test]
fn test_contact_records() {
    assert_eq!(
        parse(
            r###"# Ads.txt file for example.com:
greenadexchange.com, 12345, DIRECT, d75815a79
blueadexchange.com, XF436, DIRECT
contact=adops@example.com
contact=http://example.com/contact-us"###
        )
        .unwrap(),
        vec![
            Row::Comment("Ads.txt file for example.com:"),
            Row::Record(Record {
                domain: "greenadexchange.com",
                account_id: "12345",
                relation: Relation::Direct,
                authority_id: Some("d75815a79"),
            }),
            Row::Record(Record {
                domain: "blueadexchange.com",
                account_id: "XF436",
                relation: Relation::Direct,
                authority_id: None,
            }),
            Row::Variable(Variable {
                name: "contact",
                value: "adops@example.com",
            }),
            Row::Variable(Variable {
                name: "contact",
                value: "http://example.com/contact-us",
            }),
        ]
    );
}

#[test]
fn test_subdomain_referral() {
    assert_eq!(
        parse(
            r###"# Ads.txt file for example.com:
greenadexchange.com, 12345, DIRECT, d75815a79
blueadexchange.com, XF436, DIRECT
subdomain=divisionone.example.com"###
        )
        .unwrap(),
        vec![
            Row::Comment("Ads.txt file for example.com:"),
            Row::Record(Record {
                domain: "greenadexchange.com",
                account_id: "12345",
                relation: Relation::Direct,
                authority_id: Some("d75815a79"),
            }),
            Row::Record(Record {
                domain: "blueadexchange.com",
                account_id: "XF436",
                relation: Relation::Direct,
                authority_id: None,
            }),
            Row::Variable(Variable {
                name: "subdomain",
                value: "divisionone.example.com",
            }),
        ]
    );
}

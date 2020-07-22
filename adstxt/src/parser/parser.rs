use std::collections::HashMap;

use crate::parser;

/// Represents [`ads.txt`][`ads.txt`] data.
///
/// [`ads.txt`]: https://iabtechlab.com/ads-txt/
#[derive(Debug, PartialEq)]
pub struct AdsTxt<'a> {
    pub records: Vec<Record<'a>>,
    pub variables: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> From<Vec<Row<'a>>> for AdsTxt<'a> {
    fn from(rows: Vec<Row<'a>>) -> Self {
        let mut records = Vec::new();
        let mut variables = HashMap::new();

        for row in rows {
            match row {
                Row::Record(r) => records.push(r),
                Row::Variable(v) => variables
                    .entry(v.name)
                    .or_insert_with(Vec::new)
                    .push(v.value),
                Row::Comment(_) | Row::Blank | Row::Unknown(_) => {}
            }
        }

        AdsTxt { records, variables }
    }
}

impl<'a> std::convert::TryFrom<&'a str> for AdsTxt<'a> {
    type Error = parser::Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        parse_adstxt(s)
    }
}

/// Represents the line data of ads.txt.
#[derive(Debug, PartialEq)]
pub enum Row<'a> {
    Comment(&'a str),
    Record(Record<'a>),
    Variable(Variable<'a>),
    Blank,
    Unknown(&'a str),
}

/// Represents data of the format `<FIELD #1>, <FIELD #2>, <FIELD #3>, <FIELD #4>`.
#[derive(Debug, PartialEq)]
pub struct Record<'a> {
    /// FIELD #1: Domain name of the advertising system.
    pub domain: &'a str,
    /// FIELD #2: Publisher’s Account ID.
    pub account_id: &'a str,
    /// FIELD #3: Type of Account/Relationship.
    pub relation: Relation,
    /// FIELD #4: Certification Authority ID.
    pub authority_id: Option<&'a str>,
}

/// Type of Account/Relationship.
#[derive(Debug, PartialEq)]
pub enum Relation {
    Direct,
    Reseller,
}

impl std::str::FromStr for Relation {
    type Err = parser::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        relation(s)
    }
}

/// Represents data of the format `<VARIABLE>=<VALUE>`.
#[derive(Debug, PartialEq)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

/// Parse the contents of ads.txt.
///
/// See also [`parse`](fn.parse.html).
///
/// # Example
/// ```rust
/// # use adstxt::parser::*;
/// # use std::collections::HashMap;
///
/// assert_eq!(
///     parse_adstxt(
///         "# comment
/// placeholder.example.com, placeholder, DIRECT, placeholder
/// contact=adops@example.com
///
/// unknown"
///     )
///     .unwrap(),
///     AdsTxt {
///         records: vec![Record {
///             domain: "placeholder.example.com",
///             account_id: "placeholder",
///             relation: Relation::Direct,
///             authority_id: Some("placeholder"),
///         }],
///         variables: {
///             let mut map = HashMap::new();
///             map.insert("contact", vec!["adops@example.com"]);
///             map
///         },
///     }
/// );
/// ```
pub fn parse_adstxt<'a>(s: &'a str) -> parser::Result<AdsTxt<'a>> {
    let rows = parse(s)?;
    Ok(rows.into())
}

#[test]
fn test_parse_adstxt() {
    assert_eq!(
        parse_adstxt(
            "# Ads.txt file for example.com:
greenadexchange.com, 12345, DIRECT, d75815a79
blueadexchange.com, XF436, DIRECT
contact=adops@example.com
contact=http://example.com/contact-us
subdomain=divisionone.example.com"
        )
        .unwrap(),
        AdsTxt {
            records: vec![
                Record {
                    domain: "greenadexchange.com",
                    account_id: "12345",
                    relation: Relation::Direct,
                    authority_id: Some("d75815a79"),
                },
                Record {
                    domain: "blueadexchange.com",
                    account_id: "XF436",
                    relation: Relation::Direct,
                    authority_id: None,
                },
            ],
            variables: {
                let mut map = HashMap::new();
                map.insert(
                    "contact",
                    vec!["adops@example.com", "http://example.com/contact-us"],
                );
                map.insert("subdomain", vec!["divisionone.example.com"]);
                map
            }
        },
    )
}

/// Parse the contents of ads.txt.
///
/// The row data are mapped to [`Row`](enum.Row.html) and unparsed data are mapped to
/// [`Row::Unknown`](enum.Row.html#variant.Unknown).
///
/// # Example
/// ```rust
/// # use adstxt::parser::*;
/// assert_eq!(
///     parse(
///         "# comment
/// placeholder.example.com, placeholder, DIRECT, placeholder
/// contact=adops@example.com
///
/// unknown"
///     )
///     .unwrap(),
///     vec![
///         Row::Comment("comment"),
///         Row::Record(Record {
///             domain: "placeholder.example.com",
///             account_id: "placeholder",
///             relation: Relation::Direct,
///             authority_id: Some("placeholder"),
///         }),
///         Row::Variable(Variable {
///             name: "contact",
///             value: "adops@example.com",
///         }),
///         Row::Blank,
///         Row::Unknown("unknown"),
///     ]
/// );
/// ```
pub fn parse<'a>(s: &'a str) -> parser::Result<Vec<Row<'a>>> {
    let ret = s.lines().map(|s| row(s.trim())).collect::<Vec<_>>();
    let all_unknown = ret
        .iter()
        .all(|r| matches!(r, Row::Unknown(_) | Row::Blank));
    if !all_unknown {
        Ok(ret)
    } else {
        Err("all lines are unknown or blank".into())
    }
}

#[test]
fn test_parse() {
    assert!(parse("unknown\n\n").is_err());
    assert_eq!(parse("# comment").unwrap(), vec![Row::Comment("comment")]);
    assert_eq!(parse("\n\nplaceholder.example.com, placeholder, DIRECT, placeholder\nplaceholder.example.com, placeholder, DIRECT, placeholder\n\n").unwrap(), vec![
        Row::Blank,
        Row::Blank,
        Row::Record(Record{
            domain: "placeholder.example.com",
            account_id: "placeholder",
            relation: Relation::Direct,
            authority_id: Some("placeholder"),
        }),
        Row::Record(Record{
            domain: "placeholder.example.com",
            account_id: "placeholder",
            relation: Relation::Direct,
            authority_id: Some("placeholder"),
        }),
        Row::Blank,
    ]);
}

#[inline]
fn row(s: &str) -> Row<'_> {
    match s {
        "" => Row::Blank,
        s => {
            if let Some(c) = comment(s) {
                return Row::Comment(c);
            }

            match record(s) {
                Ok(Some(r)) => return Row::Record(r),
                Ok(None) => {}
                Err(_) => return Row::Unknown(s),
            }

            if let Some(v) = variable(s) {
                Row::Variable(v)
            } else {
                Row::Unknown(s)
            }
        }
    }
}

#[test]
fn test_row() {
    assert_eq!(row(""), Row::Blank);
    assert_eq!(row("# comment"), Row::Comment("comment"));
    assert_eq!(
        row("f1,f2,DIRECT"),
        Row::Record(Record {
            domain: "f1",
            account_id: "f2",
            relation: Relation::Direct,
            authority_id: None,
        })
    );
    assert_eq!(
        row("name=value"),
        Row::Variable(Variable {
            name: "name",
            value: "value",
        })
    );
    assert_eq!(row("unknown"), Row::Unknown("unknown"));
}

#[inline]
fn comment(s: &str) -> Option<&str> {
    if s.starts_with('#') {
        Some(s.split_at(1).1.trim())
    } else {
        None
    }
}

#[test]
fn test_comment() {
    assert_eq!(comment("# a"), Some("a"));
    assert_eq!(comment("#a"), Some("a"));
    assert_eq!(comment(""), None);
}

#[inline]
fn record<'a>(s: &'a str) -> parser::Result<Option<Record<'a>>> {
    let (domain, account_id, relation, authority_id) = match fields(s) {
        None => return Ok(None),
        Some(fields) => fields,
    };
    let relation = relation.parse()?;
    Ok(Some(Record {
        domain,
        account_id,
        relation,
        authority_id,
    }))
}

#[test]
fn test_record() {
    assert_eq!(
        record("example.com,12345,DIRECT").unwrap(),
        Some(Record {
            domain: "example.com",
            account_id: "12345",
            relation: Relation::Direct,
            authority_id: None,
        }),
    );
    assert_eq!(
        record("example.com,12345,DIRECT  ").unwrap(),
        Some(Record {
            domain: "example.com",
            account_id: "12345",
            relation: Relation::Direct,
            authority_id: None,
        }),
    );
    assert_eq!(
        record("example.com , 12345 , RESELLER , 5jyxf8k54").unwrap(),
        Some(Record {
            domain: "example.com",
            account_id: "12345",
            relation: Relation::Reseller,
            authority_id: Some("5jyxf8k54"),
        }),
    );
    assert_eq!(
        record("example.com,12345,RESELLER;extension data").unwrap(),
        Some(Record {
            domain: "example.com",
            account_id: "12345",
            relation: Relation::Reseller,
            authority_id: None,
        }),
    );
    assert_eq!(
        record("example.com , 12345 , RESELLER , 5jyxf8k54 ; extention data").unwrap(),
        Some(Record {
            domain: "example.com",
            account_id: "12345",
            relation: Relation::Reseller,
            authority_id: Some("5jyxf8k54"),
        }),
    );
}

#[inline]
fn fields(s: &str) -> Option<(&str, &str, &str, Option<&str>)> {
    let mut fields = s.split(',');

    let f1 = fields.next()?.trim();
    let f2 = fields.next()?.trim();
    let f3 = fields.next().map(|s| s.split(';').next().unwrap().trim())?;
    let f4 = fields.next().map(|s| s.split(';').next().unwrap().trim());

    Some((f1, f2, f3, f4))
}

#[test]
fn test_fields() {
    assert_eq!(fields("f1,f2,f3"), Some(("f1", "f2", "f3", None)));
    assert_eq!(fields("f1,f2,f3;ext-data"), Some(("f1", "f2", "f3", None)));
    assert_eq!(fields("f1,f2,f3,f4"), Some(("f1", "f2", "f3", Some("f4"))));
    assert_eq!(
        fields("f1,f2,f3,f4;ext-data"),
        Some(("f1", "f2", "f3", Some("f4")))
    );
}

#[inline]
fn relation(s: &str) -> parser::Result<Relation> {
    match s {
        "DIRECT" => Ok(Relation::Direct),
        "RESELLER" => Ok(Relation::Reseller),
        _ => Err("field #3 must be `DIRECT` or `RESELLER`".into()),
    }
}

#[test]
fn test_relation() {
    assert_eq!(relation("DIRECT").unwrap(), Relation::Direct);
    assert_eq!(relation("RESELLER").unwrap(), Relation::Reseller);
    assert!(relation("Relation").is_err());
}

#[inline]
fn variable(s: &str) -> Option<Variable<'_>> {
    if !s.starts_with('#') {
        if let Some(index) = char_at('=', s) {
            if index == 0 {
                None
            } else {
                let (name, last) = s.split_at(index);
                let value = last.strip_prefix('=').unwrap();
                Some(Variable { name, value })
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[test]
fn test_variable() {
    assert_eq!(variable("# abc"), None);
    assert_eq!(variable("f1,f2,f3"), None);
    assert_eq!(variable("=abc"), None);
    assert_eq!(
        variable("a=b"),
        Some(Variable {
            name: "a",
            value: "b"
        })
    );
}

#[inline]
fn char_at(a: char, s: &str) -> Option<usize> {
    let a = a as u8;
    for (index, &byte) in s.as_bytes().iter().enumerate() {
        if byte == a {
            return Some(index);
        }
    }
    None
}

#[test]
fn test_char_at() {
    assert_eq!(char_at('a', "12345"), None);
    assert_eq!(char_at('1', "12345"), Some(0));
    assert_eq!(char_at('3', "12345"), Some(2));
    assert_eq!(char_at('5', "12345"), Some(4));
}

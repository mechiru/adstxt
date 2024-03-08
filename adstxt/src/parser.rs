/// Represents [`ads.txt`][`ads.txt`] data.
///
/// [`ads.txt`]: https://iabtechlab.com/ads-txt/
#[derive(Debug, PartialEq, Clone)]
pub struct AdsTxt<'a> {
    pub records: Vec<(Record<'a>, Option<Extension<'a>>)>,
    pub variables: Vec<(Variable<'a>, Option<Extension<'a>>)>,
}

impl AdsTxt<'_> {
    /// Parse the contents of ads.txt.
    ///
    /// See also [`AdsTxt::parse_lines`].
    ///
    /// # Example
    /// ```rust
    /// # use adstxt::*;
    ///
    /// assert_eq!(
    ///     AdsTxt::parse(
    ///         "# comment
    /// placeholder.example.com, placeholder, DIRECT, placeholder # Comment
    /// contact=adops@example.com
    ///
    /// unknown"
    ///     ),
    ///     AdsTxt {
    ///         records: vec![(
    ///             Record {
    ///                 domain: "placeholder.example.com",
    ///                 account_id: "placeholder",
    ///                 relation: Relation::Direct,
    ///                 authority_id: Some("placeholder"),
    ///             },
    ///             None
    ///         )],
    ///         variables: vec![(Variable { name: "contact", value: "adops@example.com" }, None)],
    ///     }
    /// );
    /// ```
    pub fn parse(data: &'_ str) -> AdsTxt<'_> {
        let (records, variables) =
            data.split('\n').fold((Vec::new(), Vec::new()), |mut acc, x| match LineData::parse(x) {
                LineData::Record { record, extension, .. } => {
                    acc.0.push((record, extension));
                    acc
                }
                LineData::Variable { variable, extension, .. } => {
                    acc.1.push((variable, extension));
                    acc
                }
                _ => acc,
            });

        AdsTxt { records, variables }
    }

    /// Parse the contents of ads.txt.
    ///
    /// The line data are mapped to [`LineData`]. See also [`AdsTxt::parse`].
    ///
    /// # Example
    /// ```rust
    /// # use adstxt::*;
    ///
    /// assert_eq!(
    ///     AdsTxt::parse_lines(
    ///         "# comment
    /// placeholder.example.com, placeholder, DIRECT, placeholder # Comment
    /// contact=adops@example.com
    ///
    /// unknown"
    ///     ),
    ///     vec![
    ///         LineData::Comment(Comment("# comment")),
    ///         LineData::Record {
    ///             record: Record {
    ///                 domain: "placeholder.example.com",
    ///                 account_id: "placeholder",
    ///                 relation: Relation::Direct,
    ///                 authority_id: Some("placeholder"),
    ///             },
    ///             extension: None,
    ///             comment: Some(Comment("# Comment"))
    ///         },
    ///         LineData::Variable {
    ///             variable: Variable { name: "contact", value: "adops@example.com" },
    ///             extension: None,
    ///             comment: None
    ///         },
    ///         LineData::Empty,
    ///         LineData::Unknown("unknown"),
    ///     ]
    /// );
    /// ```
    pub fn parse_lines(data: &'_ str) -> Vec<LineData<'_>> {
        data.split('\n').map(|x| LineData::parse(x)).collect()
    }
}

#[test]
fn test_adstxt_parse() {
    assert_eq!(
        AdsTxt::parse(
            "# Ads.txt file for example.com:
greenadexchange.com, 12345, DIRECT, d75815a79
blueadexchange.com, XF436, DIRECT
contact=adops@example.com
contact=http://example.com/contact-us
subdomain=divisionone.example.com"
        ),
        AdsTxt {
            records: vec![
                (
                    Record {
                        domain: "greenadexchange.com",
                        account_id: "12345",
                        relation: Relation::Direct,
                        authority_id: Some("d75815a79"),
                    },
                    None
                ),
                (
                    Record {
                        domain: "blueadexchange.com",
                        account_id: "XF436",
                        relation: Relation::Direct,
                        authority_id: None,
                    },
                    None
                ),
            ],
            variables: vec![
                (Variable { name: "contact", value: "adops@example.com" }, None,),
                (Variable { name: "contact", value: "http://example.com/contact-us" }, None,),
                (Variable { name: "subdomain", value: "divisionone.example.com" }, None,),
            ],
        },
    );
}

#[test]
fn test_line_data_parse() {
    assert_eq!(LineData::parse(""), LineData::Empty);
    assert_eq!(LineData::parse("# comment"), LineData::Comment(Comment("# comment")));
    assert_eq!(LineData::parse("f1,f2,DIRECT"), LineData::Record {
        record: Record { domain: "f1", account_id: "f2", relation: Relation::Direct, authority_id: None },
        extension: None,
        comment: None
    });
    assert_eq!(LineData::parse("name=value"), LineData::Variable {
        variable: Variable { name: "name", value: "value" },
        extension: None,
        comment: None
    });
    assert_eq!(LineData::parse("unknown"), LineData::Unknown("unknown"));
}

/// Represents the line data of ads.txt.
#[derive(Debug, PartialEq, Clone)]
pub enum LineData<'a> {
    /// Comment.
    Comment(Comment<'a>),
    /// Record.
    Record { record: Record<'a>, extension: Option<Extension<'a>>, comment: Option<Comment<'a>> },
    /// Variable record.
    Variable { variable: Variable<'a>, extension: Option<Extension<'a>>, comment: Option<Comment<'a>> },
    /// Blank line.
    Empty,
    /// Unknown line data.
    Unknown(&'a str),
}

impl LineData<'_> {
    #[inline(always)]
    fn parse(line: &'_ str) -> LineData<'_> {
        match line.trim() {
            "" => LineData::Empty,
            line => {
                if let Some(comment) = parse_comment(line) {
                    LineData::Comment(comment)
                } else if let Some((record, extension, comment)) = parse_record(line) {
                    LineData::Record { record, extension, comment }
                } else if let Some((variable, extension, comment)) = parse_variable(line) {
                    LineData::Variable { variable, extension, comment }
                } else {
                    LineData::Unknown(line)
                }
            }
        }
    }
}

/// Represents comment of ads.txt.
#[derive(Debug, PartialEq, Clone)]
pub struct Comment<'a>(pub &'a str);

#[inline(always)]
fn parse_comment(line: &'_ str) -> Option<Comment<'_>> {
    if line.starts_with('#') { Some(Comment(line)) } else { None }
}

#[test]
fn test_parse_comment() {
    assert_eq!(parse_comment("# this is comment."), Some(Comment("# this is comment.")));
}

/// Represents record of ads.txt.
#[derive(Debug, PartialEq, Clone)]
pub struct Record<'a> {
    /// FIELD #1: Domain name of the advertising system.
    pub domain: &'a str,
    /// FIELD #2: Publisher's Account ID.
    pub account_id: &'a str,
    /// FIELD #3: Type of Account/Relationship.
    pub relation: Relation<'a>,
    /// FIELD #4: Certification Authority ID.
    pub authority_id: Option<&'a str>,
}

#[inline(always)]
fn parse_record_tail(line_tail: &'_ str) -> Option<(Option<Extension<'_>>, Option<Comment<'_>>)> {
    let mut iter = line_tail.chars().enumerate();
    loop {
        let (i, c) = iter.next()?;
        match c {
            ';' => {
                let start = i + 1;
                for (j, c) in iter {
                    if c == '#' {
                        return Some((
                            Some(Extension(line_tail[start..j].trim())),
                            Some(Comment(line_tail[j..].trim())),
                        ));
                    }
                }
                return Some((Some(Extension(line_tail[start..].trim())), None));
            }
            '#' => return Some((None, Some(Comment(line_tail[i..].trim())))),
            _ => {}
        }
    }
}

#[test]
fn test_parse_record_tail() {
    assert_eq!(parse_record_tail("hoge  "), None);
    assert_eq!(parse_record_tail("  "), None);
    assert_eq!(parse_record_tail(";"), Some((Some(Extension("")), None)));
    assert_eq!(parse_record_tail("  ; ext-data  "), Some((Some(Extension("ext-data")), None)));
    assert_eq!(parse_record_tail("fuga ; ext-data"), Some((Some(Extension("ext-data")), None)));
    assert_eq!(
        parse_record_tail("  ; ext-data  # comment  "),
        Some((Some(Extension("ext-data")), Some(Comment("# comment"))))
    );
    assert_eq!(parse_record_tail("#"), Some((None, Some(Comment("#")))));
    assert_eq!(parse_record_tail("# comment  "), Some((None, Some(Comment("# comment")))));
}

#[inline(always)]
fn parse_record(line: &'_ str) -> Option<(Record<'_>, Option<Extension<'_>>, Option<Comment<'_>>)> {
    let (domain, tail) = line.split_once(',').map(|x| (x.0.trim(), x.1))?;
    let (account_id, tail) = tail.split_once(',').map(|x| (x.0.trim(), x.1))?;
    let Some((relation, tail)) = tail.split_once(',').map(|x| (parse_relation(x.0.trim()), x.1)) else {
        let relation = match tail.split_once([';', '#']) {
            Some(x) => parse_relation(x.0.trim()),
            None => parse_relation(tail.trim()),
        };
        let authority_id = None;

        return Some(match parse_record_tail(tail) {
            Some((extension, comment)) => (Record { domain, account_id, relation, authority_id }, extension, comment),
            None => (Record { domain, account_id, relation, authority_id }, None, None),
        });
    };
    let authority_id = Some(match tail.split_once([';', '#']) {
        Some(x) => x.0.trim(),
        None => tail.trim(),
    });

    Some(match parse_record_tail(tail) {
        Some((extension, comment)) => (Record { domain, account_id, relation, authority_id }, extension, comment),
        None => (Record { domain, account_id, relation, authority_id }, None, None),
    })
}

#[test]
fn test_parse_record() {
    assert_eq!(
        parse_record("f1, f2, f3"),
        Some((
            Record { domain: "f1", account_id: "f2", relation: Relation::Unknown("f3"), authority_id: None },
            None,
            None
        ))
    );
    assert_eq!(
        parse_record("f1, f2, f3 ; ext-data"),
        Some((
            Record { domain: "f1", account_id: "f2", relation: Relation::Unknown("f3"), authority_id: None },
            Some(Extension("ext-data")),
            None
        ))
    );
    assert_eq!(
        parse_record("f1, f2, f3 ; ext-data # comment  "),
        Some((
            Record { domain: "f1", account_id: "f2", relation: Relation::Unknown("f3"), authority_id: None },
            Some(Extension("ext-data")),
            Some(Comment("# comment"))
        ))
    );
    assert_eq!(
        parse_record("f1, f2, f3 # comment"),
        Some((
            Record { domain: "f1", account_id: "f2", relation: Relation::Unknown("f3"), authority_id: None },
            None,
            Some(Comment("# comment"))
        ))
    );
    assert_eq!(
        parse_record("f1, f2, f3, f4"),
        Some((
            Record { domain: "f1", account_id: "f2", relation: Relation::Unknown("f3"), authority_id: Some("f4") },
            None,
            None
        ))
    );
    assert_eq!(
        parse_record("f1, f2, f3, f4 ; ext-data"),
        Some((
            Record { domain: "f1", account_id: "f2", relation: Relation::Unknown("f3"), authority_id: Some("f4") },
            Some(Extension("ext-data")),
            None
        ))
    );
    assert_eq!(
        parse_record("f1, f2, f3, f4 ; ext-data # comment  "),
        Some((
            Record { domain: "f1", account_id: "f2", relation: Relation::Unknown("f3"), authority_id: Some("f4") },
            Some(Extension("ext-data")),
            Some(Comment("# comment"))
        ))
    );
    assert_eq!(
        parse_record("f1, f2, f3, f4 # comment"),
        Some((
            Record { domain: "f1", account_id: "f2", relation: Relation::Unknown("f3"), authority_id: Some("f4") },
            None,
            Some(Comment("# comment"))
        ))
    );
}

/// Represents type of Account/Relationship.
#[derive(Debug, PartialEq, Clone)]
pub enum Relation<'a> {
    Direct,
    Reseller,
    Unknown(&'a str),
}

#[inline(always)]
fn parse_relation(s: &'_ str) -> Relation<'_> {
    match s {
        "DIRECT" => Relation::Direct,
        "RESELLER" => Relation::Reseller,
        s => Relation::Unknown(s),
    }
}

#[test]
fn test_parse_relation() {
    assert_eq!(parse_relation("DIRECT"), Relation::Direct);
    assert_eq!(parse_relation("RESELLER"), Relation::Reseller);
    assert_eq!(parse_relation("Relation"), Relation::Unknown("Relation"));
}

/// Represents variable record.
#[derive(Debug, PartialEq, Clone)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[inline(always)]
fn parse_variable(line: &'_ str) -> Option<(Variable<'_>, Option<Extension<'_>>, Option<Comment<'_>>)> {
    let (name, tail) = line.split_once('=').map(|x| (x.0.trim(), x.1))?;

    let value = match tail.split_once([';', '#']) {
        Some(x) => x.0.trim(),
        None => tail.trim(),
    };

    Some(match parse_record_tail(tail) {
        Some((extension, comment)) => (Variable { name, value }, extension, comment),
        None => (Variable { name, value }, None, None),
    })
}

#[test]
fn test_parse_variable() {
    assert_eq!(parse_variable("# abc"), None);
    assert_eq!(parse_variable("f1,f2,f3"), None);
    assert_eq!(parse_variable("=abc"), Some((Variable { name: "", value: "abc" }, None, None)));
    assert_eq!(
        parse_variable(" name = value ; ext-data"),
        Some((Variable { name: "name", value: "value" }, Some(Extension("ext-data")), None))
    );
    assert_eq!(
        parse_variable(" name = value # comment"),
        Some((Variable { name: "name", value: "value" }, None, Some(Comment("# comment"))))
    );
    assert_eq!(
        parse_variable(" name = value ; ext-data # comment"),
        Some((Variable { name: "name", value: "value" }, Some(Extension("ext-data")), Some(Comment("# comment"))))
    );
}

/// Represents extension data of ads.txt record.
#[derive(Debug, PartialEq, Clone)]
pub struct Extension<'a>(pub &'a str);

# adstxt

[![ci](https://github.com/mechiru/adstxt/workflows/ci/badge.svg)](https://github.com/mechiru/adstxt/actions?query=workflow:ci)
[![Rust Documentation](https://docs.rs/adstxt/badge.svg)](https://docs.rs/adstxt)
[![Latest Version](https://img.shields.io/crates/v/adstxt.svg)](https://crates.io/crates/adstxt)

This library provides a parser for [ads.txt v1.0.2](https://iabtechlab.com/wp-content/uploads/2019/03/IAB-OpenRTB-Ads.txt-Public-Spec-1.0.2.pdf).

## Example

```rust
# use adstxt::*;

assert_eq!(
    AdsTxt::parse(
        "# comment
placeholder.example.com, placeholder, DIRECT, placeholder # Comment
contact=adops@example.com

unknown"
    ),
    AdsTxt {
        records: vec![(
            Record {
                domain: "placeholder.example.com",
                account_id: "placeholder",
                relation: Relation::Direct,
                authority_id: Some("placeholder"),
            },
            None
        )],
        variables: vec![(Variable { name: "contact", value: "adops@example.com" }, None)],
    }
);
```

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

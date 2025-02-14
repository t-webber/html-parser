# Html Parser

# C parser

[![github](https://img.shields.io/badge/GitHub-t--webber/html--parser-blue?logo=GitHub)](https://github.com/t-webber/html-parser)
[![license](https://img.shields.io/badge/Licence-MIT-darkgreen)](https://github.com/t-webber/html-parser?tab=MIT-1-ov-file)
[![coverage](https://img.shields.io/badge/Coverage-100%25-purple)](https://github.com/t-webber/html-parser/actions/workflows/nightly.yml)
[![rust-edition](https://img.shields.io/badge/Rust--edition-2024-darkred?logo=Rust)](https://doc.rust-lang.org/stable/edition-guide/rust-2024/)

This is a rust library that lexes and parses C source files.

## Standard

This is a simple lightweight html parser, that converts an html file (in the `str` format) to a tree representing the html tags and text.

## Getting started

You can install it with

```shell
cargo add html_parser
```

then us it like this:

```rust
use html_parser::prelude::*;

let html: &str = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Html sample</title>
    </head>
    <body>
        <p>This is an html sample.</p>
    </body>
</html>
"#;

// Parse your html
let tree: Html = parse_html(html).expect("Invalid HTML");

// Now you can use it!
assert!(format!("{tree}") == html);
```

## Find & filter

You can also use the `find` and `filter` methods to manage this html. To do this, you need to create your filtering options with the `Filter` type.

### Filter

```rust
use html_parser::prelude::*;

let html: &str = r##"
  <section>
    <h1>Welcome to My Random Page</h1>
    <nav>
      <ul>
        <li><a href="/home">Home</a></li>
        <li><a href="/about">About</a></li>
        <li><a href="/services">Services</a></li>
        <li><a href="/contact">Contact</a></li>
      </ul>
    </nav>
  </section>
"##;

// Create your filter
let filter = Filter::default().tag_name("li");

// Parse your html
let filtered_tree: Html = parse_html(html).expect("Invalid HTML").filter(&filter);

// Check the result: filtered_tree contains the 4 lis from the above html string
if let Html::Vec(links) = filtered_tree {
    assert!(links.len() == 4)
} else {
    unreachable!()
}
```

### Find

The finder returns the first element that respects the filter:

```rust
use html_parser::prelude::*;

let html: &str = r##"
  <section>
    <h1>Welcome to My Random Page</h1>
    <nav>
      <ul>
        <li><a href="/home">Home</a></li>
        <li><a href="/about">About</a></li>
        <li><a href="/services">Services</a></li>
        <li><a href="/contact">Contact</a></li>
      </ul>
    </nav>
  </section>
"##;

// Create your filter
let filter = Filter::default().tag_name("a");

// Parse your html
let link: Html = parse_html(html).expect("Invalid HTML").find(&filter).expect("No `a` tags");

// Check the result: link contains `<a href="/home">Home</a>`
if let Html::Tag { tag, child, .. } = link {
    if let Html::Text(text) = *child {
        assert!(&tag.name == "a" && text == "Home");
    } else {
        unreachable!()
    }
} else {
    unreachable!()
}
```

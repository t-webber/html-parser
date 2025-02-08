use std::fs::read_to_string;

use html_parser::prelude::*;

use super::test_maker;

macro_rules! test_filter {
    ($($name:ident: $filter:expr => $expect:expr)*) => {
        $(
            #[test]
            fn $name() {
                let content = read_to_string("tests/data/index.html").unwrap();
                let tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}")).filter(&$filter);
                test_maker(stringify!($name), $expect, tree, $filter)
            }
        )*
    };
}

test_filter!(

filter_comment: Filter::default().comment(true).document(false) =>
"<!--@<li> --><!-- prettier-ignore --><!-- prettier-ignore --><!--- Table --->"

filter_doctype: Filter::default().document(true) =>
"<!><!DOCTYPE ><!DOCTYPE html>"

filter_prefix: Filter::default().attribute_value("xlink:href", "#") =>
r##"<a xlink:href="#">About</a>"##

filter_radio: Filter::default().attribute_value("type", "radio").attribute_name("radio") =>
r#"<input radio type="radio" name="radio" id="radio1" /><input radio type="radio" name="radio" id="radio2" />"#

filter_radio_id: Filter::default().attribute_value("type", "radio").attribute_value("id", "radio2") =>
r#"<input radio type="radio" name="radio" id="radio2" />"#

filter_enabled: Filter::default().attribute_name("enabled") =>
"<button enabled /><input enabled />"

filter_buttons: Filter::default().tag_name("button").tag_name("input") =>
r#"
<input type="sub\mit" id="name" name="name" />
<input type='sub"mit' value="Submit" />
<button enabled />
<input enabled />
<input type="checkbox" id="check" />
<input radio type="radio" name="radio" id="radio1" />
<input radio type="radio" name="radio" id="radio2" />
<input type="date" />
<input type="file" />
"#

filter_tr: Filter::default().tag_name("tr") =>
"<tr><th>ID</th><th>Name</th></tr><tr><td>1</td><td>Alice</td></tr><tr><td>2</td><td>Bob</td></tr>"

ul_depth: Filter::default().depth(1).tag_name("li") =>
r##"
<ul>
    <li><a xlink:href="#">About</a></li>
    <li>
        <a href="#">Contact<br> us</a>
    </li>
</ul>
"##

);

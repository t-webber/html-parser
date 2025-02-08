use std::fs::read_to_string;

use html_parser::prelude::*;

use super::test_maker;

macro_rules! test_find {
    ($($name:ident: $filter:expr => $expect:expr)*) => {
        $(
            #[test]
            fn $name() {
                let content = read_to_string("tests/data/index.html").unwrap();
                let tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}")).find(&$filter);
                if let Some(tree) = tree {
                    test_maker(stringify!($name), $expect, tree, $filter)
                } else {
                    panic!("No matching element corresponding to the given filters.")
                }
            }
        )*
    };
}

test_find!(

find_section: Filter::default().all().tag_name("section") =>
r##"<section>
        <h2>Forms</h2>
        <form action="#" method="post">
            <input type="sub\mit" id="name" name="name" />
            <input type='sub"mit' value="Submit" />
            <!-- prettier-ignore -->
            <button enabled/>
        </form>
    </section>"##

);

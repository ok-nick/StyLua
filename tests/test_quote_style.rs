use stylua_lib::{format_code, Config, QuoteStyle};

fn format(input: &str, quote_style: QuoteStyle) -> String {
    format_code(input, Config::default().with_quote_style(quote_style), None).unwrap()
}

#[test]
#[cfg_attr(feature = "luau", ignore)]
fn test_auto_prefer_double_quotes() {
    insta::assert_snapshot!(
        format(
            r###"
local a = "foobar"
local c = 'foobar'
local d = "foo\"bar"
local e = 'foo"bar'
local f = "foo'bar"
local g = 'foo\'bar'
local h = "foo\"bar'baz"
local i = 'foo"bar\'baz'
            "###,
            QuoteStyle::AutoPreferDouble
        ),
        @r###"
    local a = "foobar"
    local c = "foobar"
    local d = 'foo"bar'
    local e = 'foo"bar'
    local f = "foo'bar"
    local g = "foo'bar"
    local h = "foo\"bar'baz"
    local i = "foo\"bar'baz"
    "###
    );
}

#[test]
#[cfg_attr(feature = "luau", ignore)]
fn test_auto_prefer_single_quotes() {
    insta::assert_snapshot!(
        format(
            r###"
local a = "foobar"
local c = 'foobar'
local d = "foo\"bar"
local e = 'foo"bar'
local f = "foo'bar"
local g = 'foo\'bar'
local h = "foo\"bar'baz"
local i = 'foo"bar\'baz'
            "###,
            QuoteStyle::AutoPreferSingle
        ),
        @r###"
    local a = 'foobar'
    local c = 'foobar'
    local d = 'foo"bar'
    local e = 'foo"bar'
    local f = "foo'bar"
    local g = "foo'bar"
    local h = 'foo"bar\'baz'
    local i = 'foo"bar\'baz'
    "###
    );
}

#[test]
#[cfg_attr(feature = "luau", ignore)]
fn test_force_double_quotes() {
    insta::assert_snapshot!(
        format(
            r###"
local a = "foobar"
local c = 'foobar'
local d = "foo\"bar"
local e = 'foo"bar'
local f = "foo'bar"
local g = 'foo\'bar'
local h = "foo\"bar'baz"
local i = 'foo"bar\'baz'
            "###,
            QuoteStyle::ForceDouble
        ),
        @r###"
    local a = "foobar"
    local c = "foobar"
    local d = "foo\"bar"
    local e = "foo\"bar"
    local f = "foo'bar"
    local g = "foo'bar"
    local h = "foo\"bar'baz"
    local i = "foo\"bar'baz"
    "###
    );
}

#[test]
#[cfg_attr(feature = "luau", ignore)]
fn test_force_single_quotes() {
    insta::assert_snapshot!(
        format(
            r###"
local a = "foobar"
local c = 'foobar'
local d = "foo\"bar"
local e = 'foo"bar'
local f = "foo'bar"
local g = 'foo\'bar'
local h = "foo\"bar'baz"
local i = 'foo"bar\'baz'
            "###,
            QuoteStyle::ForceSingle
        ),
        @r###"
    local a = 'foobar'
    local c = 'foobar'
    local d = 'foo"bar'
    local e = 'foo"bar'
    local f = 'foo\'bar'
    local g = 'foo\'bar'
    local h = 'foo"bar\'baz'
    local i = 'foo"bar\'baz'
    "###
    );
}

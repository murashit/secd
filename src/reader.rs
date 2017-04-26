use combine::*;
use compiler::Ast;

pub fn read(input: &str) -> Result<(Vec<Ast>, &str), ParseError<&str>> {
    parser(whitespace)
        .with(many1(parser(expression).skip(parser(whitespace))))
        .skip(eof())
        .parse(input.trim())
}

fn whitespace<I>(input: I) -> ParseResult<(), I>
    where I: Stream<Item = char>
{
    let comment = token(';')
        .and(skip_many(satisfy(|c| c != '\n')))
        .map(|_| ());
    skip_many(skip_many1(char::space()).or(comment)).parse_stream(input)
}

fn expression<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    between(token('('), token(')'), parser(list))
        .or(parser(atom))
        .or(parser(quote))
        .or(parser(quasiquote))
        .or(try(parser(unquote)).or(parser(unquote_splicing)))
        .parse_stream(input)
}

fn atom<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    try(parser(integer))
        .or(parser(symbol))
        .or(parser(hash))
        .parse_stream(input)
}

fn hash<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    token('#')
        .with(any())
        .map(|c| match c {
                 't' => Ast::Boolean(true),
                 'f' => Ast::Boolean(false),
                 _ => unimplemented!(),
             })
        .parse_stream(input)
}

fn integer<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    let unsigned = many1::<String, _>(char::digit());
    let positive = unsigned
        .to_owned()
        .map(|s| Ast::Integer(s.parse::<u32>().unwrap() as i32));
    let negative = token('-')
        .with(unsigned)
        .map(|s| Ast::Integer(-(s.parse::<u32>().unwrap() as i32)));
    negative.or(positive).parse_stream(input)
}

fn symbol<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    let special_initial = one_of("!$%&*/:<=>?^_~+-".chars());
    let peculiar_identifier = one_of("+-".chars());
    let initial = char::letter().or(special_initial);
    let special_subsequent = one_of("+-.@".chars());
    let subsequent = initial
        .to_owned()
        .or(char::digit())
        .or(special_subsequent);
    initial
        .and(many(subsequent))
        .map(|(i, s): (char, String)| Ast::Symbol(format!("{}{}", i, s)))
        .or(peculiar_identifier.map(|s: char| Ast::Symbol(s.to_string())))
        .parse_stream(input)
}

fn quote<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    token('\'')
        .with(parser(expression))
        .map(|val| Ast::new_list(&[Ast::new_symbol("quote"), val], Ast::Nil))
        .parse_stream(input)
}

fn quasiquote<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    token('`')
        .with(parser(expression))
        .map(|val| Ast::new_list(&[Ast::new_symbol("quasiquote"), val], Ast::Nil))
        .parse_stream(input)
}

fn unquote<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    token(',')
        .with(parser(expression))
        .map(|val| Ast::new_list(&[Ast::new_symbol("unquote"), val], Ast::Nil))
        .parse_stream(input)
}

fn unquote_splicing<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    token(',')
        .with(token('@'))
        .with(parser(expression))
        .map(|val| Ast::new_list(&[Ast::new_symbol("unquote-splicing"), val], Ast::Nil))
        .parse_stream(input)
}

fn list<I>(input: I) -> ParseResult<Ast, I>
    where I: Stream<Item = char>
{
    let former = char::spaces().with(many(parser(expression).skip(char::spaces())));
    let dotted = token('.').skip(char::spaces()).with(parser(expression));
    let nil = char::spaces().map(|_| Ast::Nil);
    former
        .and(dotted.or(nil))
        .map(|(former, last): (Vec<_>, _)| Ast::new_list(&former, last))
        .parse_stream(input)
}

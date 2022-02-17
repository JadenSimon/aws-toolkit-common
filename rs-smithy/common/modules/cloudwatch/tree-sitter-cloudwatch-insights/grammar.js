// no regex flag support, so we use this
function caseInsensitive (keyword) {
    return new RegExp(keyword
      .split('')
      .map(letter => `[${letter}${letter.toUpperCase()}]`)
      .join('')
    )
  }

module.exports = grammar({
    name: 'cloudwatch_insights',
    // I'm hoping whitespace doesn't matter
    // I wonder if tree sitter actually runs the code or if it just parses the file ?

    extras: $ => [
        $.comment,
        /[\s\p{Zs}\uFEFF\u2060\u200B]/, // whitespace
    ],

    //supertypes: $ => [$.statement],
    
    word: $ => $._word,

    rules: {
        source_file: $ => repeat(seq($._statement, optional($._statement_separator))),
        _word: $ => /[A-Za-z]+/,

        _statement: $ => choice(
            $.display_statement, 
            $.fields_statement,
            $.stats_statement,
            $.filter_statement,
            $.sort_statement,
            $.limit_statement,
            $.parse_statement,
        ),
        _statement_separator: $ => '|',

        _expression_sequence: $ => seq($._expression, repeat(seq(',', $._expression))),
        _identifier_sequence: $ => seq($.identifier, repeat(seq(',', $.identifier))),

        // commands (statement keywords)
        // note that display should only occur once in a sequence of statements, though we won't check for that
        display_statement: $ => seq('display', $._identifier_sequence),
        // note that only _some_ expressions are valid in some statements, but we can chck for that later
        fields_statement: $ => seq('fields', $._expression_sequence),
        filter_statement: $ => seq('filter', $._expression_sequence),
        sort_statement: $ => seq('sort', $._identifier_sequence, optional($.sort_order)),
        stats_statement: $ => seq('stats', choice($._expression_sequence, alias($._as_expression, $.binary_expression), alias($._by_expression, $.by_expression))),
        limit_statement: $ => seq('limit', $.number_literal),
        parse_statement: $ => seq('parse', $.identifier, choice($.regexp_literal, $._parse_string)),

        // we are assuming that parsing with 'globs' requires using 'as'
        _parse_string: $ => seq($.string_literal, $.variadic_alias),


        // Every field but alphanumeric ones need backticks (@field is an exception)
        _field_key: $ => /[@A-Za-z0-9]+/, // TODO: get actual alphanumeric regexp
        // should we really separate out @[field_name] ? could be useful to parse
        _nested_field: $ => prec.right(2, seq(choice($.identifier, $._nested_field), '.', $.identifier)),
        //_dotted_field: $ => prec.right(seq(optional('@'), repeat1(seq($._field_key, optional('.'))))),
        _escaped_field: $ => /`.*`/,
        

        sort_order: $ => choice('asc', 'desc'),

        // copy-pasta
        // it'd be nice if we support named captures e.g. (?<foo>...)
        regexp_pattern: $ => token.immediate(prec(-1,
            repeat1(choice(
                seq(
                '[',
                repeat(choice(
                    seq('\\', /./), // escaped character
                    /[^\]\n\\]/       // any character besides ']' or '\n'
                )),
                ']'
                ),              // square-bracket-delimited character class
                seq('\\', /./), // escaped character
                /[^/\\\[\n]/    // any character besides '[', '\', '/', '\n'
            ))
        )),

        // copy-paste
        _unescaped_double_string_fragment: $ => token.immediate(prec(1, /[^"\\]+/)),
  
        // same here
        _unescaped_single_string_fragment: $ => token.immediate(prec(1, /[^'\\]+/)),

        _string: $ => choice(
            seq(
              '"',
              repeat(choice(
                alias($._unescaped_double_string_fragment, $.string_fragment),
                token.immediate('\\'),
              )),
              '"'
            ),
            seq(
              "'",
              repeat(choice(
                alias($._unescaped_single_string_fragment, $.string_fragment),
                token.immediate('\\'),
              )),
              "'"
            )
        ),


        string_literal: $ => $._string,
        number_literal: $ => /[0-9]+/,
        regexp_literal: $ => seq('/', $.regexp_pattern, token.immediate('/')), // TODO: regexp flags are added like so `(?i)Exp...`
        array_literal: $ => seq('[', repeat(seq(choice($.string_literal, $.number_literal), optional(','))), ']'),
        _literal: $ => choice($.string_literal, $.regexp_literal, $.number_literal, $.array_literal), 

        identifier: $ => choice($._field_key, $._nested_field, $._escaped_field),

        parenthesized_expression: $ => seq('(', repeat1(seq($._expression, optional(','))), ')'),
        _expression: $ => choice($.parenthesized_expression, $._secondary_expression),
        _secondary_expression: $ => choice($._literal, $.call_expression, $.identifier, $.binary_expression),
        // `not` must come before a predicate (such as above)


        unary_expression: $ => {
            const operators = [
                'not',
            ]

            return choice(...operators.map((op, i) => 
                prec.left(i, seq(
                    field('operator', op),
                    field('argument', $._expression),
                ))
            ))
        },

        // These are 'special-case' binary operators where the right side is restricted
        // as [identifier, ...] <-- can be variadic in `parse`, but in `fields` it cannot!
        // ~= [regexp]
        // like [regexp]
        // in [array_literal] <-- not sure if this has to be a literal ?
        // by [expression, ...]
        variadic_alias: $ => seq(field('operator', 'as'), $._identifier_sequence),

        // should come before 'as' i.e. (foo by bar) as baz
        // only used for 'by' in 'stats'
        // this is so bad...
        // TODO: add 'statement_modifier' just for 'by' + 'stats' to workaround this 
        _as_expression: $ => prec.left(20, seq(
            field('left', alias($._by_expression, $.by_expression)),
            field('operator', 'as'), 
            field('right', $.identifier),
        )),
        _by_expression: $ => prec.left(19, seq(
            field('left', $._expression),
            field('operator', 'by'), 
            field('right', choice($._expression, alias($._expression_sequence, $.expression_sequence))),
        )),

        binary_expression: $ => {
            const operators = [
                'and', // logical always has lowest left associativity
                'or', // TODO: does this come before or after and?
                '=',
                '!=',
                '<',
                '<=',
                '>',
                '>=',
                '~=', // pattern
                'like', // eq. to ~=
                /not\s+like/, // hmm
                '+',
                '-',
                '*',
                '/',
                '^',
                '%',
                'as',
                'in',
                /not\s+in/, // why did they do this??
            ]

            return choice(...operators.map((op, i) => 
                prec.left(i, seq(
                    field('left', $._expression),
                    field('operator', op),
                    field('right', $._expression),
                ))
            ))
        },

        // note that '*' is considered syntatically valid for all functions, though semantically it is only valid for `count`
        arguments: $ => seq('(', optional(choice('*', $._expression_sequence)), ')'),
        call_expression: $ => prec(4, seq($.identifier, $.arguments)),

        // is `logGroup` an intrinsic?

        // extras
        comment: $ => /#.*/,
    }
});

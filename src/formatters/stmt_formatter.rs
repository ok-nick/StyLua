use crate::formatters::{
    trivia_formatter::{
        strip_trivia, FormatTriviaType, UpdateLeadingTrivia, UpdateTrailingTrivia, UpdateTrivia,
    },
    trivia_util, CodeFormatter, EndTokenType,
};
use full_moon::ast::{Do, ElseIf, FunctionCall, GenericFor, If, NumericFor, Repeat, Stmt, While};
use full_moon::node::Node;
use full_moon::tokenizer::{Token, TokenReference, TokenType};

macro_rules! fmt_stmt {
    ($fmter:expr, $value:ident, { $($(#[$inner:meta])* $operator:ident = $output:ident,)+ }) => {
        match $value {
            $(
                $(#[$inner])*
                Stmt::$operator(stmt) => Stmt::$operator($fmter.$output(stmt)),
            )+
            other => panic!("unknown node {:?}", other),
        }
    };
}

impl CodeFormatter {
    /// Format a Do node
    pub fn format_do_block<'ast>(&self, do_block: &Do<'ast>) -> Do<'ast> {
        // Create trivia
        let additional_indent_level =
            self.get_range_indent_increase(CodeFormatter::get_token_range(do_block.do_token()));
        let leading_trivia =
            FormatTriviaType::Append(vec![self.create_indent_trivia(additional_indent_level)]);
        let trailing_trivia = FormatTriviaType::Append(vec![self.create_newline_trivia()]);

        let do_token = crate::fmt_symbol!(self, do_block.do_token(), "do")
            .update_trivia(leading_trivia.to_owned(), trailing_trivia.to_owned());
        let end_token = self
            .format_end_token(do_block.end_token(), EndTokenType::BlockEnd)
            .update_trivia(leading_trivia, trailing_trivia);

        do_block
            .to_owned()
            .with_do_token(do_token)
            .with_end_token(end_token)
    }

    /// Format a GenericFor node
    pub fn format_generic_for<'ast>(&mut self, generic_for: &GenericFor<'ast>) -> GenericFor<'ast> {
        // Create trivia
        let additional_indent_level =
            self.get_range_indent_increase(CodeFormatter::get_token_range(generic_for.for_token()));
        let leading_trivia = vec![self.create_indent_trivia(additional_indent_level)];
        let mut trailing_trivia = vec![self.create_newline_trivia()];

        let for_token = crate::fmt_symbol!(self, generic_for.for_token(), "for ")
            .update_leading_trivia(FormatTriviaType::Append(leading_trivia.to_owned()));
        let (formatted_names, mut names_comments_buf) = self.format_punctuated(
            generic_for.names(),
            &CodeFormatter::format_token_reference_mut,
        );

        #[cfg(feature = "luau")]
        let type_specifiers = generic_for
            .type_specifiers()
            .map(|x| match x {
                Some(type_specifier) => Some(self.format_type_specifier(type_specifier)),
                None => None,
            })
            .collect();

        let in_token = crate::fmt_symbol!(self, generic_for.in_token(), " in ");
        let (formatted_expr_list, mut expr_comments_buf) =
            self.format_punctuated(generic_for.expressions(), &CodeFormatter::format_expression);

        // Create comments buffer and append to end of do token
        names_comments_buf.append(&mut expr_comments_buf);
        // Append trailing trivia to the end
        names_comments_buf.append(&mut trailing_trivia);

        let do_token = crate::fmt_symbol!(self, generic_for.do_token(), " do")
            .update_trailing_trivia(FormatTriviaType::Append(names_comments_buf));

        let end_token = self
            .format_end_token(generic_for.end_token(), EndTokenType::BlockEnd)
            .update_trivia(
                FormatTriviaType::Append(leading_trivia),
                FormatTriviaType::Append(vec![self.create_newline_trivia()]), // trailing_trivia was emptied when it was appended to names_comment_buf
            );

        let generic_for = generic_for
            .to_owned()
            .with_for_token(for_token)
            .with_names(formatted_names)
            .with_in_token(in_token)
            .with_expressions(formatted_expr_list)
            .with_do_token(do_token)
            .with_end_token(end_token);
        #[cfg(feature = "luau")]
        let generic_for = generic_for.with_type_specifiers(type_specifiers);
        generic_for
    }

    /// Formats an ElseIf node - This must always reside within format_if
    fn format_else_if<'ast>(&mut self, else_if_node: &ElseIf<'ast>) -> ElseIf<'ast> {
        // Calculate trivia
        let additional_indent_level = self.get_range_indent_increase(
            CodeFormatter::get_token_range(else_if_node.else_if_token()),
        );
        let leading_trivia = vec![self.create_indent_trivia(additional_indent_level)];
        let trailing_trivia = vec![self.create_newline_trivia()];

        // Determine if we need to hang the condition
        let last_line_str_len = (strip_trivia(else_if_node.else_if_token()).to_string()
            + &strip_trivia(else_if_node.condition()).to_string()
            + &strip_trivia(else_if_node.then_token()).to_string())
            .len()
            + 2; // Include space before and after condition
        let indent_spacing =
            (self.indent_level + additional_indent_level.unwrap_or(0)) * self.config.indent_width;
        let require_multiline_expression = (indent_spacing + last_line_str_len)
            > self.config.column_width
            || trivia_util::expression_contains_inline_comments(else_if_node.condition());

        let (else_if_trailing_trivia, then_text) = if require_multiline_expression {
            (vec![self.create_newline_trivia()], "then")
        } else {
            (vec![Token::new(TokenType::spaces(1))], " then")
        };

        let formatted_else_if_token = self
            .format_end_token(else_if_node.else_if_token(), EndTokenType::BlockEnd)
            .update_leading_trivia(FormatTriviaType::Append(leading_trivia.to_owned()))
            .update_trailing_trivia(FormatTriviaType::Append(else_if_trailing_trivia));

        let formatted_condition = if require_multiline_expression {
            // Add the expression list into the indent range, as it will be indented by one
            let expr_range = else_if_node
                .condition()
                .range()
                .expect("no range for else if condition");
            self.add_indent_range((expr_range.0.bytes(), expr_range.1.bytes()));

            let condition = self.format_expression(else_if_node.condition());
            self.hang_expression(condition, additional_indent_level, None)
                .update_leading_trivia(FormatTriviaType::Append(vec![
                    self.create_indent_trivia(Some(additional_indent_level.unwrap_or(0) + 1))
                ]))
        } else {
            self.format_expression(else_if_node.condition())
        };

        let formatted_then_token = crate::fmt_symbol!(self, else_if_node.then_token(), then_text)
            .update_trivia(
                if require_multiline_expression {
                    FormatTriviaType::Append(leading_trivia)
                } else {
                    FormatTriviaType::NoChange
                },
                FormatTriviaType::Append(trailing_trivia),
            );

        else_if_node
            .to_owned()
            .with_else_if_token(formatted_else_if_token)
            .with_condition(formatted_condition)
            .with_then_token(formatted_then_token)
    }

    /// Format an If node
    pub fn format_if<'ast>(&mut self, if_node: &If<'ast>) -> If<'ast> {
        // Calculate trivia
        let additional_indent_level =
            self.get_range_indent_increase(CodeFormatter::get_token_range(if_node.if_token()));
        let leading_trivia = vec![self.create_indent_trivia(additional_indent_level)];
        let trailing_trivia = vec![self.create_newline_trivia()];

        // Determine if we need to hang the condition
        let last_line_str_len = (strip_trivia(if_node.if_token()).to_string()
            + &strip_trivia(if_node.condition()).to_string()
            + &strip_trivia(if_node.then_token()).to_string())
            .len()
            + 2; // Include space before and after condition
        let indent_spacing =
            (self.indent_level + additional_indent_level.unwrap_or(0)) * self.config.indent_width;
        let require_multiline_expression = (indent_spacing + last_line_str_len)
            > self.config.column_width
            || trivia_util::expression_contains_inline_comments(if_node.condition());

        let (if_text, then_text) = if require_multiline_expression {
            ("if\n", "then")
        } else {
            ("if ", " then")
        };

        let formatted_if_token = crate::fmt_symbol!(self, if_node.if_token(), if_text)
            .update_leading_trivia(FormatTriviaType::Append(leading_trivia.to_owned()));

        let formatted_condition = if require_multiline_expression {
            // Add the expression list into the indent range, as it will be indented by one
            let expr_range = if_node
                .condition()
                .range()
                .expect("no range for if condition");
            self.add_indent_range((expr_range.0.bytes(), expr_range.1.bytes()));

            let condition = self.format_expression(if_node.condition());
            self.hang_expression(condition, additional_indent_level, None)
                .update_leading_trivia(FormatTriviaType::Append(vec![
                    self.create_indent_trivia(Some(additional_indent_level.unwrap_or(0) + 1))
                ]))
        } else {
            self.format_expression(if_node.condition())
        };

        let formatted_then_token = crate::fmt_symbol!(self, if_node.then_token(), then_text)
            .update_trivia(
                if require_multiline_expression {
                    FormatTriviaType::Append(leading_trivia.to_owned())
                } else {
                    FormatTriviaType::NoChange
                },
                FormatTriviaType::Append(trailing_trivia.to_owned()),
            );
        let formatted_end_token = self
            .format_end_token(if_node.end_token(), EndTokenType::BlockEnd)
            .update_trivia(
                FormatTriviaType::Append(leading_trivia.to_owned()),
                FormatTriviaType::Append(trailing_trivia.to_owned()),
            );

        let formatted_else_if = match if_node.else_if() {
            Some(else_if) => Some(
                else_if
                    .iter()
                    .map(|else_if| self.format_else_if(else_if))
                    .collect(),
            ),
            None => None,
        };

        let formatted_else_token = match if_node.else_token() {
            Some(token) => {
                let formatted = self
                    .format_end_token(token, EndTokenType::BlockEnd)
                    .update_trivia(
                        FormatTriviaType::Append(leading_trivia),
                        FormatTriviaType::Append(trailing_trivia),
                    );
                Some(formatted)
            }
            None => None,
        };

        if_node
            .to_owned()
            .with_if_token(formatted_if_token)
            .with_condition(formatted_condition)
            .with_then_token(formatted_then_token)
            .with_else_if(formatted_else_if)
            .with_else_token(formatted_else_token)
            .with_end_token(formatted_end_token)
    }

    /// Format a NumericFor node
    pub fn format_numeric_for<'ast>(&mut self, numeric_for: &NumericFor<'ast>) -> NumericFor<'ast> {
        // Create trivia
        let additional_indent_level =
            self.get_range_indent_increase(CodeFormatter::get_token_range(numeric_for.for_token()));
        let leading_trivia = vec![self.create_indent_trivia(additional_indent_level)];
        let trailing_trivia = vec![self.create_newline_trivia()];

        let for_token = crate::fmt_symbol!(self, numeric_for.for_token(), "for ")
            .update_leading_trivia(FormatTriviaType::Append(leading_trivia.to_owned()));
        let formatted_index_variable = self.format_token_reference(numeric_for.index_variable());

        #[cfg(feature = "luau")]
        let type_specifier = match numeric_for.type_specifier() {
            Some(type_specifier) => Some(self.format_type_specifier(type_specifier)),
            None => None,
        };

        let equal_token = crate::fmt_symbol!(self, numeric_for.equal_token(), " = ");
        let formatted_start_expression = self.format_expression(numeric_for.start());
        let start_end_comma = crate::fmt_symbol!(self, numeric_for.start_end_comma(), ", ");
        let formatted_end_expression = self.format_expression(numeric_for.end());

        let (end_step_comma, formatted_step_expression) = match numeric_for.step() {
            Some(step) => (
                Some(crate::fmt_symbol!(
                    self,
                    numeric_for.end_step_comma().unwrap(),
                    ", "
                )),
                Some(self.format_expression(step)),
            ),
            None => (None, None),
        };

        let do_token = crate::fmt_symbol!(self, numeric_for.do_token(), " do")
            .update_trailing_trivia(FormatTriviaType::Append(trailing_trivia.to_owned()));
        let end_token = self
            .format_end_token(numeric_for.end_token(), EndTokenType::BlockEnd)
            .update_trivia(
                FormatTriviaType::Append(leading_trivia),
                FormatTriviaType::Append(trailing_trivia),
            );

        let numeric_for = numeric_for
            .to_owned()
            .with_for_token(for_token)
            .with_index_variable(formatted_index_variable)
            .with_equal_token(equal_token)
            .with_start(formatted_start_expression)
            .with_start_end_comma(start_end_comma)
            .with_end(formatted_end_expression)
            .with_end_step_comma(end_step_comma)
            .with_step(formatted_step_expression)
            .with_do_token(do_token)
            .with_end_token(end_token);
        #[cfg(feature = "luau")]
        let numeric_for = numeric_for.with_type_specifier(type_specifier);

        numeric_for
    }

    /// Format a Repeat node
    pub fn format_repeat_block<'ast>(&mut self, repeat_block: &Repeat<'ast>) -> Repeat<'ast> {
        // Calculate trivia
        let additional_indent_level = self
            .get_range_indent_increase(CodeFormatter::get_token_range(repeat_block.repeat_token()));
        let leading_trivia = vec![self.create_indent_trivia(additional_indent_level)];
        let trailing_trivia = vec![self.create_newline_trivia()];

        let repeat_token = crate::fmt_symbol!(self, repeat_block.repeat_token(), "repeat")
            .update_trivia(
                FormatTriviaType::Append(leading_trivia.to_owned()),
                FormatTriviaType::Append(trailing_trivia.to_owned()),
            );
        let until_token = crate::fmt_symbol!(self, repeat_block.until_token(), "until ")
            .update_leading_trivia(FormatTriviaType::Append(leading_trivia.to_owned()));

        // Determine if we need to hang the condition
        let last_line_str_len = (strip_trivia(repeat_block.until_token()).to_string()
            + &strip_trivia(repeat_block.until()).to_string())
            .len()
            + 1; // Include space before until and condition

        let indent_spacing =
            (self.indent_level + additional_indent_level.unwrap_or(0)) * self.config.indent_width;
        let require_multiline_expression = (indent_spacing + last_line_str_len)
            > self.config.column_width
            || trivia_util::expression_contains_inline_comments(repeat_block.until());

        let formatted_until = self.format_expression(repeat_block.until());
        let formatted_until_trivia = match require_multiline_expression {
            true => {
                // Add the expression list into the indent range, as it will be indented by one
                let expr_range = repeat_block
                    .until()
                    .range()
                    .expect("no range for repeat until");
                self.add_indent_range((expr_range.0.bytes(), expr_range.1.bytes()));
                self.hang_expression(formatted_until, additional_indent_level, None)
            }
            false => {
                formatted_until.update_trailing_trivia(FormatTriviaType::Append(trailing_trivia))
            }
        };

        repeat_block
            .to_owned()
            .with_repeat_token(repeat_token)
            .with_until_token(until_token)
            .with_until(formatted_until_trivia)
    }

    /// Format a While node
    pub fn format_while_block<'ast>(&mut self, while_block: &While<'ast>) -> While<'ast> {
        // Calculate trivia
        let additional_indent_level = self
            .get_range_indent_increase(CodeFormatter::get_token_range(while_block.while_token()));
        let leading_trivia = vec![self.create_indent_trivia(additional_indent_level)];
        let trailing_trivia = vec![self.create_newline_trivia()];

        // Determine if we need to hang the condition
        let last_line_str = strip_trivia(while_block.while_token()).to_string()
            + &strip_trivia(while_block.condition()).to_string()
            + &strip_trivia(while_block.do_token()).to_string();
        let last_line_str_len = last_line_str.len() + 2; // Include space before and after condition

        let indent_spacing =
            (self.indent_level + additional_indent_level.unwrap_or(0)) * self.config.indent_width;
        let require_multiline_expression = (indent_spacing + last_line_str_len)
            > self.config.column_width
            || trivia_util::expression_contains_inline_comments(while_block.condition());

        let (while_text, do_text) = if require_multiline_expression {
            ("while\n", "do")
        } else {
            ("while ", " do")
        };

        let while_token = crate::fmt_symbol!(self, while_block.while_token(), while_text)
            .update_leading_trivia(FormatTriviaType::Append(leading_trivia.to_owned()));

        let formatted_condition = if require_multiline_expression {
            // Add the expression list into the indent range, as it will be indented by one
            let expr_range = while_block
                .condition()
                .range()
                .expect("no range for while condition");
            self.add_indent_range((expr_range.0.bytes(), expr_range.1.bytes()));

            let condition = self.format_expression(while_block.condition());
            self.hang_expression(condition, additional_indent_level, None)
                .update_leading_trivia(FormatTriviaType::Append(vec![
                    self.create_indent_trivia(Some(additional_indent_level.unwrap_or(0) + 1))
                ]))
        } else {
            self.format_expression(while_block.condition())
        };

        let do_token = crate::fmt_symbol!(self, while_block.do_token(), do_text).update_trivia(
            if require_multiline_expression {
                FormatTriviaType::Append(leading_trivia.to_owned())
            } else {
                FormatTriviaType::NoChange
            },
            FormatTriviaType::Append(trailing_trivia.to_owned()),
        );

        let end_token = self
            .format_end_token(while_block.end_token(), EndTokenType::BlockEnd)
            .update_trivia(
                FormatTriviaType::Append(leading_trivia),
                FormatTriviaType::Append(trailing_trivia),
            );

        while_block
            .to_owned()
            .with_while_token(while_token)
            .with_condition(formatted_condition)
            .with_do_token(do_token)
            .with_end_token(end_token)
    }

    /// Wrapper around `format_function_call`, but also handles adding the trivia around the function call.
    /// This can't be done in the original function, as function calls are not always statements, but can also be
    /// in expressions.
    pub fn format_function_call_stmt<'ast>(
        &mut self,
        function_call: &FunctionCall<'ast>,
    ) -> FunctionCall<'ast> {
        // Calculate trivia
        let additional_indent_level = self
            .get_range_indent_increase(CodeFormatter::get_range_in_prefix(function_call.prefix()));
        let leading_trivia = vec![self.create_indent_trivia(additional_indent_level)];
        let trailing_trivia = vec![self.create_newline_trivia()];

        self.format_function_call(function_call).update_trivia(
            FormatTriviaType::Append(leading_trivia),
            FormatTriviaType::Append(trailing_trivia),
        )
    }

    pub fn format_stmt<'ast>(&mut self, stmt: &Stmt<'ast>) -> Stmt<'ast> {
        crate::check_should_format!(self, stmt);

        fmt_stmt!(self, stmt, {
            Assignment = format_assignment,
            Do = format_do_block,
            FunctionCall = format_function_call_stmt,
            FunctionDeclaration = format_function_declaration,
            GenericFor = format_generic_for,
            If = format_if,
            LocalAssignment = format_local_assignment,
            LocalFunction = format_local_function,
            NumericFor = format_numeric_for,
            Repeat = format_repeat_block,
            While = format_while_block,
            #[cfg(feature = "luau")] CompoundAssignment = format_compound_assignment,
            #[cfg(feature = "luau")] ExportedTypeDeclaration = format_exported_type_declaration,
            #[cfg(feature = "luau")] TypeDeclaration = format_type_declaration_stmt,
            #[cfg(feature = "lua52")] Goto = format_goto,
            #[cfg(feature = "lua52")] Label = format_label,
        })
    }
}

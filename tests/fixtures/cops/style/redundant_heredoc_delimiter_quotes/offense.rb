do_something(<<~'EOS')
             ^^^^^^^^ Style/RedundantHeredocDelimiterQuotes: Remove the redundant heredoc delimiter quotes, use `<<~EOS` instead.
  no string interpolation style text
EOS

do_something(<<-"EOS")
             ^^^^^^^^ Style/RedundantHeredocDelimiterQuotes: Remove the redundant heredoc delimiter quotes, use `<<-EOS` instead.
  plain text here
EOS

do_something(<<"EOS")
             ^^^^^^^ Style/RedundantHeredocDelimiterQuotes: Remove the redundant heredoc delimiter quotes, use `<<EOS` instead.
  just plain text
EOS

process(<<~'END', option: { setting: '\u0031-\u0039' }, verbose: true)
        ^^^^^^^^ Style/RedundantHeredocDelimiterQuotes: Remove the redundant heredoc delimiter quotes, use `<<~END` instead.
  line one
  line two
END

run_code(<<~'RUBY').should == "result\n"
         ^^^^^^^^ Style/RedundantHeredocDelimiterQuotes: Remove the redundant heredoc delimiter quotes, use `<<~RUBY` instead.
  puts "hello".encoding
RUBY

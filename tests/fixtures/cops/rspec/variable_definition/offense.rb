let('user_name') { 'Adam' }
    ^^^^^^^^^^^ RSpec/VariableDefinition: Use symbols for variable names.
let('email') { 'test@example.com' }
    ^^^^^^^ RSpec/VariableDefinition: Use symbols for variable names.
let!('count') { 42 }
     ^^^^^^^ RSpec/VariableDefinition: Use symbols for variable names.

# Mail DSL subject with string arg — no block on the subject call itself,
# but RuboCop still flags it if inside an example group context.
Mail.new do
  subject 'testing message delivery'
          ^^^^^^^^^^^^^^^^^^^^^^^^^^ RSpec/VariableDefinition: Use symbols for variable names.
end

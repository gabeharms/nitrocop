badVariable = 1
^^^^^^^^^^^ Naming/VariableName: Use snake_case for variable names.

myValue = 2
^^^^^^^ Naming/VariableName: Use snake_case for variable names.

firstName = "John"
^^^^^^^^^ Naming/VariableName: Use snake_case for variable names.

@badVariable = 1
^^^^^^^^^^^^ Naming/VariableName: Use snake_case for variable names.

@myValue = 2
^^^^^^^^ Naming/VariableName: Use snake_case for variable names.

@@badVariable = 1
^^^^^^^^^^^^^ Naming/VariableName: Use snake_case for variable names.

def foo(badParam)
        ^^^^^^^^ Naming/VariableName: Use snake_case for variable names.
end

def bar(ok, badName:)
            ^^^^^^^^ Naming/VariableName: Use snake_case for variable names.
end

firstArg = "foo"
^^^^^^^^ Naming/VariableName: Use snake_case for variable names.
do_something(firstArg)
             ^^^^^^^^ Naming/VariableName: Use snake_case for variable names.

items.each do |itemName|
               ^^^^^^^^ Naming/VariableName: Use snake_case for variable names.
end

[1, 2].map { |numVal| numVal }
              ^^^^^^ Naming/VariableName: Use snake_case for variable names.
                      ^^^^^^ Naming/VariableName: Use snake_case for variable names.

_myLocal = 1
^^^^^^^^ Naming/VariableName: Use snake_case for variable names.

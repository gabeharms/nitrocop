def foo # rubocop:disable Metrics/CyclomaticComplexity
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/DisableCopsWithinSourceCodeDirective: RuboCop disable/enable directives are not permitted.
end

def bar # rubocop:enable Metrics/CyclomaticComplexity
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/DisableCopsWithinSourceCodeDirective: RuboCop disable/enable directives are not permitted.
end

def baz # rubocop:enable all
        ^^^^^^^^^^^^^^^^^^^^ Style/DisableCopsWithinSourceCodeDirective: RuboCop disable/enable directives are not permitted.
end

# rubocop:disable Metrics/MethodLength
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/DisableCopsWithinSourceCodeDirective: RuboCop disable/enable directives are not permitted.
def long_method
  puts "a"
end
# rubocop:enable Metrics/MethodLength
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/DisableCopsWithinSourceCodeDirective: RuboCop disable/enable directives are not permitted.

# rubocop:todo Metrics/AbcSize
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Style/DisableCopsWithinSourceCodeDirective: RuboCop disable/enable directives are not permitted.
def complex_method
  x = 1
end

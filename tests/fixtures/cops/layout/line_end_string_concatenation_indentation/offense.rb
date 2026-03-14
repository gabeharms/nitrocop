text = 'hello' \
    'world'
    ^^^^^^^ Layout/LineEndStringConcatenationIndentation: Align parts of a string concatenated with backslash.

result = "one" \
           "two"
           ^^^^^ Layout/LineEndStringConcatenationIndentation: Align parts of a string concatenated with backslash.

# In always-indented context (def body), second line should be indented
def some_method
  'x' \
  'y' \
  ^^^ Layout/LineEndStringConcatenationIndentation: Indent the first part of a string concatenated with backslash.
  'z'
end

# In always-indented context (block body), second line should be indented
foo do
  "hello" \
  "world"
  ^^^^^^^ Layout/LineEndStringConcatenationIndentation: Indent the first part of a string concatenated with backslash.
end

# In always-indented context (lambda body), second line should be indented
x = ->(obj) {
  "line one" \
  "line two"
  ^^^^^^^^^^ Layout/LineEndStringConcatenationIndentation: Indent the first part of a string concatenated with backslash.
}

# Operator assignment inside def - NOT always-indented, parent is op_asgn
def update_record
  msg = "initial"
  msg +=
    "first part " \
      "second part"
      ^^^^^^^^^^^^^ Layout/LineEndStringConcatenationIndentation: Align parts of a string concatenated with backslash.
end

# Index operator assignment inside def - NOT always-indented
def process_errors
  errors[:detail] +=
    "a valid item has a single " \
      "root directory"
      ^^^^^^^^^^^^^^^^ Layout/LineEndStringConcatenationIndentation: Align parts of a string concatenated with backslash.
end

# Call operator write (x.y +=) inside def
def handle_response
  response.body +=
    "extra content " \
      "appended here"
      ^^^^^^^^^^^^^^^ Layout/LineEndStringConcatenationIndentation: Align parts of a string concatenated with backslash.
end

# Or-assignment inside def
def set_default
  @value ||=
    "default " \
      "value"
      ^^^^^^^ Layout/LineEndStringConcatenationIndentation: Align parts of a string concatenated with backslash.
end

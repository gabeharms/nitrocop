array = ['Item 1' 'Item 2']
         ^^^^^^^^^^^^^^^^^^ Lint/ImplicitStringConcatenation: Combine 'Item 1' and 'Item 2' into a single string literal, rather than using implicit string concatenation.
x = "foo" "bar"
    ^^^^^^^^^^^ Lint/ImplicitStringConcatenation: Combine "foo" and "bar" into a single string literal, rather than using implicit string concatenation.
y = "hello" "world"
    ^^^^^^^^^^^^^^^ Lint/ImplicitStringConcatenation: Combine "hello" and "world" into a single string literal, rather than using implicit string concatenation.
z = ["first" "second", "third"]
     ^^^^^^^^^^^^^^^^ Lint/ImplicitStringConcatenation: Combine "first" and "second" into a single string literal, rather than using implicit string concatenation.
w = ['a' 'b' 'c']
     ^^^^^^ Lint/ImplicitStringConcatenation: Combine 'a' and 'b' into a single string literal, rather than using implicit string concatenation.
         ^^^^^^ Lint/ImplicitStringConcatenation: Combine 'b' and 'c' into a single string literal, rather than using implicit string concatenation.
ber = "\xa4\x2d" \
  "\x04\x0b" "objectclass"
  ^^^^^^^^^^^^^^^^^^^^^^^^^ Lint/ImplicitStringConcatenation: Combine "\x04\x0b" and "objectclass" into a single string literal, rather than using implicit string concatenation.

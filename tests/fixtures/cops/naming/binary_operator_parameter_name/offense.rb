def +(x)
      ^ Naming/BinaryOperatorParameterName: When defining the `+` operator, name its argument `other`.
end
def -(foo)
      ^^^ Naming/BinaryOperatorParameterName: When defining the `-` operator, name its argument `other`.
end
def ==(val)
       ^^^ Naming/BinaryOperatorParameterName: When defining the `==` operator, name its argument `other`.
end

# Without parentheses around the parameter
def + date_time
      ^^^^^^^^^ Naming/BinaryOperatorParameterName: When defining the `+` operator, name its argument `other`.
  date_time
end
def - date_time
      ^^^^^^^^^ Naming/BinaryOperatorParameterName: When defining the `-` operator, name its argument `other`.
  date_time
end
def == val
       ^^^ Naming/BinaryOperatorParameterName: When defining the `==` operator, name its argument `other`.
  val
end

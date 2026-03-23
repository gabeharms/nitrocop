class A
  @@test = 10
  ^^^^^^ Style/ClassVars: Replace class var @@test with a class instance var.
end

class B
  @@count = 0
  ^^^^^^^ Style/ClassVars: Replace class var @@count with a class instance var.
end

class C
  @@name = "test"
  ^^^^^^ Style/ClassVars: Replace class var @@name with a class instance var.
end

class D
  @@a, @@b = 1, 2
  ^^^ Style/ClassVars: Replace class var @@a with a class instance var.
       ^^^ Style/ClassVars: Replace class var @@b with a class instance var.
end

class E
  @@x, @@y, @@z = nil, nil, nil
  ^^^ Style/ClassVars: Replace class var @@x with a class instance var.
       ^^^ Style/ClassVars: Replace class var @@y with a class instance var.
            ^^^ Style/ClassVars: Replace class var @@z with a class instance var.
end

class F
  local_var, @@cvar = foo, bar
             ^^^^^^ Style/ClassVars: Replace class var @@cvar with a class instance var.
end

x = { a: 1,
      b: 2,
      c: 3 }
y = { d: 4,
      e: 5 }
z = { f: 6,
      g: 7 }

# Separator/value alignment: extra spaces after colon (key style)
hash1 = {
  a: 0,
  bb: 1,
}

# Separator/value alignment: hash rockets with extra spaces
hash2 = {
  'ccc' => 2,
  'dddd' => 3
}

# First pair with bad spacing (even first pair gets checked for separator/value)
hash3 = {
  :a => 0,
  :bb => 1,
}

# Mixed offenses: key misalignment AND separator/value spacing
hash4 = { :a => 0,
          :bb => 1,
          :ccc => 2 }

# Keyword splat alignment
{foo: 'bar',
 **extra
}

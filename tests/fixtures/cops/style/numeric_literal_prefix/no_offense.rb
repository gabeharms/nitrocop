num = 0o1234
num = 0x12AB
num = 0b10101
num = 1234
num = 100
num = 0
# Underscore-separated decimals starting with 0 are NOT octal
num = 0_30
num = 1_00
num = 0_50
# Complex/rational number literals should NOT be flagged (042i = Complex(0,34))
num = 042i
num = -042i
num = 042r

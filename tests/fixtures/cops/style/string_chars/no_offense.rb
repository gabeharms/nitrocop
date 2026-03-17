string.chars
string.split(/ /)
string.split
string.split(',')
string.split('ab')

# Regex with flags should not be flagged (//u is Unicode mode, semantically different from //)
string.split(//u)
string.split(//i)
string.split(//m)

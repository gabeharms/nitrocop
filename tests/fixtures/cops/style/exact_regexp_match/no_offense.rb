string =~ /\Astring.*\z/
string === /\A0+\z/
string =~ /^string$/
string =~ /\Astring\z/i
string == 'exact'
match(/\Astring\z/)

# Empty exact match - no literal between anchors
string =~ /\A\z/

# Non-literal escape sequences
string =~ /\Ahello\nworld\z/
string =~ /\A\tindented\z/
string =~ /\Aline\rend\z/

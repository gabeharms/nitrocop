x = 1
x == ""
x != y
a => "hello"
{a: 1, b: 2}
x += 1
"hello=world"
# x=1 inside comment
x = "a==b"

# Default parameters (handled by SpaceAroundEqualsInParameterDefault)
def foo(bar=1)
end
def baz(x=1, y=2)
end

# Spaceship operator (<=>) should not trigger => check
x <=> y
[1, 2, 3].sort { |a, b| a <=> b }

# Operator method definitions should not be flagged
def ==(other)
  id == other.id
end

def !=(other)
  !(self == other)
end

def []=(key, value)
  @data[key] = value
end

def <=>(other)
  name <=> other.name
end

def self.===(other)
  other.is_a?(self)
end

def >=(other)
  value >= other.value
end

# Safe navigation with operator method: &.!=
table_name&.!= node.left.relation.name

# Method call with dot before operator
x.== y

# Binary operators with proper spacing
x + y
x - y
x * y
x / y
x % y
x & y
x | y
x ^ y
x << y
x >> 1
x && y
x || y
x < y
x > y
x <= y
x >= y
x <=> y

# Unary operators (not binary — should not be flagged)
z = -x
z = +x

# Exponent operator with no_space style (default) should not be flagged
x = 2**10
y = n**(k - 1)

# AllowForAlignment: operators aligned across adjacent lines
title  = data[:title]  || ''
url    = data[:url]    || ''
width  = data[:width]  || 0
height = data[:height] || 0

# Trailing spaces before comment after operator — not flagged
x ||  # fallback
  y
a &&  # condition check
  b

# Operator at start of line (continuation) — indentation, not extra spacing
result = foo \
  + bar
x = a \
    || b

# Compound assignments with proper spacing
x += 1
y -= 2
z *= 3
a /= 4
b %= 5
c ||= 0
d &&= true
e **= 2
f <<= 1
g >>= 1
h ^= 0xff
i |= 0x01
j &= 0xff

# Match operators with proper spacing
x =~ /abc/
y !~ /abc/

# Class inheritance with proper spacing
class Foo < Bar
end

# Singleton class with proper spacing
class << self
end

# Rescue => with proper spacing
begin
rescue Exception => e
end

# Triple equals with proper spacing
Hash === z

# Setter call with proper spacing
x.y = 2

# Ternary operator with proper spacing
x == 0 ? 1 : 2

# Rational literal (no_space style default for /)
x = 2/3r

# Ranges should not be flagged
a, b = (1..2), (1...3)

# Scope operator should not be flagged
Zlib::GzipWriter

# Operator symbols should not be flagged
func(:-)

# Tabs around operator are acceptable
a =	1
x	= 1
y	=	2
'000'	=>	'General error'
'001' =>	'3D Not authenticated'
x ==	y
x	!= y

# Cross-operator alignment: ||= aligned with = (same end column)
PATH_PATTERN           = /^\/\w+/
PROTOCOL_PATTERN       = /^\w+:\/\//
README                 = File.dirname(__FILE__) + '/../../README.md'
@output              ||= STDOUT

# Cross-operator alignment: += aligned with = (same end column)
x  = 1
y += 2

# Cross-operator alignment: various compound operators aligned
found        += items
total        += count
status      ||= 0

# Hash with multi-byte UTF-8 keys aligned by => (curly quotes are 3 bytes each)
# Must not flag any of these as "extra space" around =>
rewrites = {
  'should amass debt'                    => 'amasses debt',
  'should echo the input'                => 'echoes the input',
  "shouldn\u2019t return something"      => 'does not return something',
  "SHOULDN\u2019T BE true"               => 'IS NOT true',
}

# Alignment across blank line (RuboCop accepts this)
expected = posts(:welcome)

tagging  = Tagging.all.merge!(includes: :taggable).find(taggings(:welcome_general).id)

# Alignment when extra space exists and adjacent line has same = alignment
@integer_message = 12345
@output  = StringIO.new
@logger  = Logger.new(@output)

# Alignment with different operators ending with = (e.g., var.foo = a aligned with var.bar != b)
var.foo       = a
var.bar      != b
var.quux     <= c
var.garply   >= d
var.corge    == e
var.fred     += f
var.baz     === g

# Hash rocket alignment (extra space after => for alignment)
{
  1 =>  2,
  11 => 3
}

# Operator at start of continuation line (no space before +, -)
result = first_value +
  second_value
x = a ||
  b

# Alignment with a blank line between assignments (RuboCop accepts this)
expected = posts(:welcome)

tagging2  = Tagging.all

# Alignment: setter methods and other = operators at same column
unit  = rate_detail.showback_unit
rate  = rate_detail.rate
type  = rate_detail.type

# Alignment across non-assignment lines at different indentation
a  = 1
foo(bar)
b  = 2

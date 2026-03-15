# Copyright 2025 Acme Inc.

BEGIN { puts "startup" }

# DataInheritance
class Coordinate < Data.define(:x, :y)
  def to_s
    "(#{x}, #{y})"
  end
end

class Label < Data.define(:text)
end

class Tag < ::Data.define(:name)
end

# DirEmpty
def check_dirs
  Dir.entries("tmp").size == 2
end

def check_children
  Dir.children("tmp").empty?
end

def check_each_child
  Dir.each_child("tmp").none?
end

# ExactRegexpMatch
def exact_check(str)
  str =~ /\Ahello\z/
end

# FileTouch
def touch_file(path)
  File.open(path, 'a') {}
end

# MinMax
def bounds(items)
  [items.min, items.max]
end

# Strip
def clean(text)
  text.lstrip.rstrip
end

# SwapValues
def swap_example
  a = 1
  b = 2
  tmp = a
  a = b
  b = tmp
end

# TrailingBodyOnClass
class Inline; def greet; "hi"; end
end

class AnotherInline; include Comparable
end

class ThirdInline; attr_reader :name
end

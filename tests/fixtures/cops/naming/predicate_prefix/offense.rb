def has_value?
    ^^^^^^^^^^ Naming/PredicatePrefix: Rename `has_value?` to `value?`.
  !@value.nil?
end

def is_valid
    ^^^^^^^^ Naming/PredicatePrefix: Rename `is_valid` to `valid?`.
  @valid
end

def has_children?
    ^^^^^^^^^^^^^ Naming/PredicatePrefix: Rename `has_children?` to `children?`.
  @children.any?
end

# Singleton methods should also be checked (alias on_defs on_def)
def self.is_active
         ^^^^^^^^^ Naming/PredicatePrefix: Rename `is_active` to `active?`.
  @active
end

def self.has_permission
         ^^^^^^^^^^^^^^ Naming/PredicatePrefix: Rename `has_permission` to `permission?`.
  @permission
end

# is_attr? should be flagged when prefix is in ForbiddenPrefixes
# (prefix should be stripped even though ? is already present)
def is_attr?
    ^^^^^^^^ Naming/PredicatePrefix: Rename `is_attr?` to `attr?`.
  @attr
end

def have_items
    ^^^^^^^^^^ Naming/PredicatePrefix: Rename `have_items` to `items?`.
  @items.any?
end

def does_match
    ^^^^^^^^^^ Naming/PredicatePrefix: Rename `does_match` to `match?`.
  @match
end

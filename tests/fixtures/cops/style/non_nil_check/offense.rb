x != nil
^^^^^^^^ Style/NonNilCheck: Prefer `!x.nil?` over `x != nil`.

foo != nil
^^^^^^^^^^ Style/NonNilCheck: Prefer `!foo.nil?` over `foo != nil`.

bar.baz != nil
^^^^^^^^^^^^^^ Style/NonNilCheck: Prefer `!bar.baz.nil?` over `bar.baz != nil`.

# != nil inside a compound expression in a predicate method is still an offense
def expired?(time = Time.now)
  expires_at != nil && time > expires_at
  ^^^^^^^^^^^^^^^^^^^ Style/NonNilCheck: Prefer `!expires_at.nil?` over `expires_at != nil`.
end

def maxed?
  @max != nil && @stores.length == @max
  ^^^^^^^^^^^^^ Style/NonNilCheck: Prefer `!@max.nil?` over `@max != nil`.
end

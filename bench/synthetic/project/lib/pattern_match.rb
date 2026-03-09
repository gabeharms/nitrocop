# Copyright 2025 Acme Inc.

class PatternMatch
  # MultilineInPatternThen
  def categorize(value)
    case value
    in { type: "admin" } then
      :admin
    in { type: "user" } then
      :user
    in { type: "guest" } then
      :guest
    end
  end
end

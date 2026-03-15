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

  # InPatternThen (semicolon instead of then/newline)
  def classify(value)
    case value
    in Integer; :number
    in String; :text
    in Symbol; :symbol
    end
  end

  # DuplicateMatchPattern
  def check(value)
    case value
    in [1, 2]
      :first
    in [3, 4]
      :second
    in [1, 2]
      :duplicate
    end
  end
end

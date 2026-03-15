# Copyright 2025 Acme Inc.

class LintExtras
  # BigDecimalNew
  def make_decimal(val)
    BigDecimal.new(val, 3)
  end

  # CircularArgumentReference
  def greet(name: name)
    puts name
  end

  def cook(dry_ingredients = dry_ingredients)
    dry_ingredients.reduce(&:+)
  end

  def bake(pie: pie)
    pie.heat_up
  end

  # EachWithObjectArgument
  def bad_accumulator(items)
    items.each_with_object(0) { |item, acc| acc + item }
  end

  # NextWithoutAccumulator
  def sum_even(numbers)
    numbers.reduce(0) do |acc, n|
      next if n.odd?
      acc + n
    end
  end

  # NumericOperationWithConstantResult (only fires at top level — see below)

  # RedundantRegexpQuantifiers
  def match_pattern(text)
    text.match?(/(?:a+)+/)
  end

  # RequireRangeParentheses
  # (commented out — this needs multi-line which is hard to trigger reliably in synthetic)

  # RescueType
  def safe_call
    do_something
  rescue nil
    nil
  end

  # SafeNavigationWithEmpty
  def check_empty(collection)
    return unless collection.items&.empty?
    "empty"
  end

  # UnescapedBracketInRegexp
  def has_bracket(text)
    text.match?(/abc]123/)
  end

  # UselessRuby2Keywords
  ruby2_keywords def no_splat; end

  ruby2_keywords def with_regular_arg(arg); end

  ruby2_keywords def with_kwargs(**opts); end
end

# ConstantOverwrittenInRescue
begin
  something
rescue => StandardError
end

begin
  something
rescue => RuntimeError
end

begin
  something
rescue => MyError
end

# NumericOperationWithConstantResult (top-level only, requires --enable-pending-cops)
# RequireRangeParentheses (pending cop, requires baseline config)
# Both are exercised in CI where the baseline config enables all pending cops.
# Locally they show as "not exercised" which is expected.

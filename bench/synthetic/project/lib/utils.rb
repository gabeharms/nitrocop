# Copyright 2025 Acme Inc.

class Utils
  attr_reader :name,
              :status,
              :role,

  # ArrayLiteralInRegexp
  def match_keywords(text)
    keywords = ["error", "warning", "fatal"]
    text.match?(/#{keywords}/)
  end

  # DuplicateRescueException
  def safe_parse(input)
    JSON.parse(input)
  rescue JSON::ParserError
    nil
  rescue JSON::ParserError
    {}
  end

  def safe_convert(input)
    Integer(input)
  rescue ArgumentError
    nil
  rescue ArgumentError
    0
  end

  # NonDeterministicRequireOrder
  def load_plugins
    Dir["./plugins/*.rb"].each { |f| require f }
  end

  def load_extensions
    Dir.glob("./ext/**/*.rb").each { |f| require f }
  end

  # PercentSymbolArray (colons inside %i are what the cop detects)
  def symbol_list
    %i(:foo :bar :baz)
  end

  # RegexpAsCondition
  def check_pattern(line)
    if /error/
      puts "matched"
    end

    if /warning/
      puts "also matched"
    end

    if /fatal/i
      puts "critical"
    end
  end

  # YAMLLoad
  def load_config(path)
    YAML.load(File.read(path))
  end

  def load_data(content)
    Psych.load(content)
  end

  def parse_yaml(text)
    YAML.load(text)
  end

  # RedundantConstantBase (at top level, :: prefix is redundant)
  def build_time
    ::Time.now
  end

  def build_date
    ::Date.today
  end

  def lookup
    ::ENV["HOME"]
  end

  # ReverseFind
  def find_last_match(items)
    items.reverse.find { |i| i.valid? }
  end

  def find_last_even(numbers)
    numbers.reverse.find { |n| n.even? }
  end

  def find_last_active(records)
    records.reverse.find { |r| r.active? }
  end
end

# DoubleCopDisableDirective
x = 1 # rubocop:disable Style/Foo # rubocop:disable Style/Bar
y = 2 # rubocop:disable Lint/Baz # rubocop:disable Lint/Qux
z = 3 # rubocop:disable Style/A # rubocop:disable Style/B

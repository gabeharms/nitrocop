# Properly aligned string concatenation
msg = "foo" \
      "bar"

text = 'hello' \
       'world'

result = "one" \
         "two"

simple = "no continuation"

# Backslash inside heredoc should not be flagged
x = <<~SQL
  SELECT * FROM users \
  WHERE id = 1
SQL

y = <<~SHELL
  echo "hello" \
    "world"
SHELL

# Properly indented in def body (always_indented context)
def some_method
  'x' \
    'y' \
    'z'
end

# Properly indented in block body (always_indented context)
foo do
  "hello" \
    "world"
end

# Aligned inside method call argument (non-indented context)
describe "should not send a notification if a notification service is not" \
         "configured" do
end

# Aligned in case/when branch (when is NOT always-indented parent)
def message
  case style
  when :first_column_after_left_parenthesis
    'Indent the right bracket the same as the first position ' \
    'after the preceding left parenthesis.'
  end
end

# Properly indented in lambda body (always_indented context, like block)
x = ->(obj) {
  "line one" \
    "line two" \
    "line three"
}

# Aligned operator assignment inside def (not always-indented)
def update_record
  msg = "initial"
  msg +=
    "first part " \
    "second part"
end

# Aligned index operator write inside def
def process_errors
  errors[:detail] +=
    "a valid item " \
    "description"
end

# Properly indented dstr inside if branch within a + concatenation in a def
# (previously a false positive — StatementsNode pass-through bug)
def library_name
  "update " +
    if items.one?
      "#{items.first} requirement " \
        "#{old_version}" \
        "to #{new_version}"
    else
      names = items.map(&:name).uniq
      if names.one?
        "requirements for #{names.first}"
      else
        "#{names[0..-2].join(', ')} and #{names[-1]}"
      end
    end
end

# Indented dstr inside if/else branch assigned with += (always-indented: parent is :if)
def build_intro
  msg = "Updates dependency"
  msg += if items.count > 2
           " #{links[0..-2].join(', ')} " \
             "and #{links[-1]}. "
         else
           " ancestor dependency #{links[1]}. "
         end
  msg
end

# Indented dstr inside elsif branch (if is always-indented parent)
def application_name
  "bump " +
    if items.one?
      record = items.first
      "#{record.name} " \
        "#{from_msg(record.version)}" \
        "to #{record.new_version}"
    elsif updating_property?
      record = items.first
      "#{prop_name} " \
        "#{from_msg(record.version)}" \
        "to #{record.new_version}"
    else
      names = items.map(&:name).uniq
      names.first
    end
end

# Indented dstr inside block within if branch
def metadata_info
  items.map do |dep|
    msg = if dep.removed?
            "\nRemoves `#{dep.name}`\n"
          else
            "\nUpdates `#{dep.name}` " \
              "#{from_msg(dep.version)}" \
              "to #{dep.new_version}"
          end
    msg
  end
end

# Indented dstr inside conditional assignment after if/else
def render_message
  msg = ""
  msg +=
    if records.count > 10
      "- Additional items viewable in " \
        "[compare view](#{url})\n"
    else
      "- See full diff in [compare view](#{url})\n"
    end
  msg
end

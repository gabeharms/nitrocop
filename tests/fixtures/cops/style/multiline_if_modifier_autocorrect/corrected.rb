if cond
  {
    result: run
  }
end

unless cond
  {
    result: run
  }
end

if value.nil?
  raise "Value checking error" \
      ' for attribute'
end
